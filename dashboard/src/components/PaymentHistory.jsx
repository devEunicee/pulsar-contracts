import React, { useEffect, useState } from 'react';

const API = process.env.REACT_APP_INDEXER_URL || 'http://localhost:3001';

export default function PaymentHistory({ merchantAddress }) {
  const [payments, setPayments] = useState([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!merchantAddress) return;
    setLoading(true);
    fetch(`${API}/events/payment_processed`)
      .then(r => r.json())
      .then(data => setPayments(data.filter(e =>
        JSON.stringify(e.topics).includes(merchantAddress)
      )))
      .finally(() => setLoading(false));
  }, [merchantAddress]);

  if (loading) return <p aria-live="polite">Loading payments…</p>;

  return (
    <section aria-labelledby="payments-heading">
      <h2 id="payments-heading">Payment History</h2>
      {payments.length === 0 ? (
        <p>No payments found.</p>
      ) : (
        <div role="region" aria-label="Payment history table" style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead>
              <tr>
                <th scope="col">Ledger</th>
                <th scope="col">Tx Hash</th>
                <th scope="col">Event</th>
                <th scope="col">Date</th>
              </tr>
            </thead>
            <tbody>
              {payments.map(p => (
                <tr key={p.id}>
                  <td>{p.ledger}</td>
                  <td style={{ fontFamily: 'monospace', fontSize: '0.8em' }}>
                    {p.tx_hash.slice(0, 12)}…
                  </td>
                  <td>{p.event_type}</td>
                  <td>{new Date(p.created_at).toLocaleString()}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}
