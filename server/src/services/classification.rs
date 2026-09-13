use std::collections::BTreeSet;

use crate::{
    mime_parser::ParsedMailSnapshotV1,
    models::{ClassificationRule, Mailbox, MessageStateUpdateRequest, RuleCondition},
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClassificationOutcome {
    pub label_ids: Vec<i64>,
    pub state: MessageStateUpdateRequest,
}

pub fn classify(
    rules: &[ClassificationRule],
    envelope_to: &str,
    snapshot: &ParsedMailSnapshotV1,
) -> ClassificationOutcome {
    let mut labels = BTreeSet::new();
    let mut state = MessageStateUpdateRequest::default();
    for rule in rules.iter().filter(|rule| rule.enabled) {
        if !rule
            .conditions
            .iter()
            .all(|condition| condition_matches(condition, envelope_to, snapshot))
        {
            continue;
        }
        labels.extend(rule.actions.add_label_ids.iter().copied());
        if let Some(archive) = rule.actions.archive {
            state.mailbox = Some(if archive {
                Mailbox::Archive
            } else {
                Mailbox::Inbox
            });
        }
        if let Some(read) = rule.actions.mark_read {
            state.read = Some(read);
        }
        if let Some(starred) = rule.actions.star {
            state.starred = Some(starred);
        }
        if let Some(trashed) = rule.actions.trash {
            state.trashed = Some(trashed);
        }
        if rule.stop_processing {
            break;
        }
    }
    ClassificationOutcome {
        label_ids: labels.into_iter().collect(),
        state,
    }
}

fn condition_matches(
    condition: &RuleCondition,
    envelope_to: &str,
    snapshot: &ParsedMailSnapshotV1,
) -> bool {
    let values = field_values(&condition.field, envelope_to, snapshot);
    values
        .iter()
        .any(|value| value_matches(value, &condition.operator, &condition.value))
}

fn field_values(field: &str, envelope_to: &str, snapshot: &ParsedMailSnapshotV1) -> Vec<String> {
    match field {
        "envelope_to" => vec![envelope_to.to_string()],
        "from" => snapshot
            .from
            .iter()
            .map(|address| address.email().to_string())
            .collect(),
        "to" => snapshot
            .to
            .iter()
            .map(|address| address.email().to_string())
            .collect(),
        "subject" => vec![snapshot.subject.clone()],
        value if value.starts_with("header:") => {
            let name = value.trim_start_matches("header:");
            snapshot
                .headers
                .iter()
                .filter(|header| header.name.eq_ignore_ascii_case(name))
                .map(|header| header.value.clone())
                .collect()
        }
        _ => Vec::new(),
    }
}

fn value_matches(actual: &str, operator: &str, expected: &str) -> bool {
    let actual = actual.trim().to_lowercase();
    let expected = expected.trim().to_lowercase();
    match operator {
        "equals" => actual == expected,
        "contains" => actual.contains(&expected),
        "starts_with" => actual.starts_with(&expected),
        "ends_with" => actual.ends_with(&expected),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RuleActions, RuleCondition};

    #[test]
    fn exact_recipient_rules_do_not_implicitly_group_plus_addresses() {
        let snapshot = crate::mime_parser::parse_message(
            b"From: sender@example.com\r\nTo: app+one@example.com\r\nSubject: Notice\r\n\r\nBody",
        )
        .unwrap()
        .snapshot;
        let rule = ClassificationRule {
            id: 1,
            name: "exact alias".into(),
            enabled: true,
            priority: 10,
            stop_processing: false,
            conditions: vec![RuleCondition {
                field: "envelope_to".into(),
                operator: "equals".into(),
                value: "app@example.com".into(),
            }],
            actions: RuleActions {
                add_label_ids: vec![7],
                ..Default::default()
            },
        };
        assert!(
            classify(&[rule], "app+one@example.com", &snapshot)
                .label_ids
                .is_empty()
        );
    }

    #[test]
    fn matching_rules_merge_labels_and_honor_stop_processing() {
        let snapshot = crate::mime_parser::parse_message(
            b"From: alerts@example.com\r\nTo: me@example.com\r\nSubject: Build failed\r\n\r\nBody",
        )
        .unwrap()
        .snapshot;
        let make_rule = |id, label, stop| ClassificationRule {
            id,
            name: format!("rule {id}"),
            enabled: true,
            priority: 10 - id,
            stop_processing: stop,
            conditions: vec![RuleCondition {
                field: "subject".into(),
                operator: "contains".into(),
                value: "failed".into(),
            }],
            actions: RuleActions {
                add_label_ids: vec![label],
                star: Some(true),
                ..Default::default()
            },
        };
        let outcome = classify(
            &[make_rule(1, 3, true), make_rule(2, 4, false)],
            "me@example.com",
            &snapshot,
        );
        assert_eq!(outcome.label_ids, vec![3]);
        assert_eq!(outcome.state.starred, Some(true));
    }
}
