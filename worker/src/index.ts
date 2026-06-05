export type Env = {
  INBOUND_URL: string;
  CF_ACCESS_CLIENT_ID: string;
  CF_ACCESS_CLIENT_SECRET: string;
};

type EnvelopeMetadata = {
  from: string;
  to: string;
  headers: Record<string, string>;
};

export default {
  async email(message: ForwardableEmailMessage, env: Env): Promise<void> {
    console.log('Received email', {
      from: message.from,
      to: message.to,
      subject: message.headers.get('subject') ?? '',
      'message-id': message.headers.get('message-id') ?? '',
      date: message.headers.get('date') ?? '',
    });

    const form = new FormData();
    const raw = await new Response(message.raw).arrayBuffer();
    const metadata: EnvelopeMetadata = {
      from: message.from,
      to: message.to,
      headers: {
        'message-id': message.headers.get('message-id') ?? '',
        subject: message.headers.get('subject') ?? '',
        date: message.headers.get('date') ?? '',
      },
    };

    form.set('raw_mime', new File([raw], 'message.eml', { type: 'message/rfc822' }));
    form.set('metadata', JSON.stringify(metadata));

    console.log('Forwarding email to', env.INBOUND_URL, {
      rawSize: raw.byteLength,
      metadataSize: JSON.stringify(metadata).length,
    });

    const response = await fetch(env.INBOUND_URL, {
      method: 'POST',
      headers: {
        'CF-Access-Client-Id': env.CF_ACCESS_CLIENT_ID,
        'CF-Access-Client-Secret': env.CF_ACCESS_CLIENT_SECRET,
      },
      body: form,
    });

    if (!response.ok) {
      const body = await response.text().catch(() => '');
      console.error('Inbound request failed', {
        status: response.status,
        statusText: response.statusText,
        body,
      });
      throw new Error(`Inbound request failed with status ${response.status}`);
    }

    console.log('Inbound request succeeded', { status: response.status });
  },
};
