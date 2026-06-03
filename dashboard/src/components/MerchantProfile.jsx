import React from 'react';

export default function MerchantProfile({ publicKey }) {
  return (
    <section aria-labelledby="profile-heading">
      <h2 id="profile-heading">Merchant Profile</h2>
      <dl>
        <dt>Wallet Address</dt>
        <dd style={{ fontFamily: 'monospace', wordBreak: 'break-all' }}>
          {publicKey || '—'}
        </dd>
      </dl>
      <p>
        To update your merchant profile (name, description, contact info), use the
        Stellar CLI or the Pulsar SDK to call <code>register_merchant</code>.
      </p>
    </section>
  );
}
