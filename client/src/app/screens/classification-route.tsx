import { LabelManager } from '../../components/classification/label-manager';
import { RuleManager } from '../../components/classification/rule-manager';
import { SavedViewManager } from '../../components/classification/saved-view-manager';

export function ClassificationRoute() {
  return (
    <section class="flex flex-col gap-4">
      <div>
        <h1 class="text-xl font-semibold">分类设置</h1>
        <p class="text-sm text-zinc-500">管理持久标签、智能视图和新邮件规则。</p>
      </div>
      <LabelManager />
      <SavedViewManager />
      <RuleManager />
    </section>
  );
}
