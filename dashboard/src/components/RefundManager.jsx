import React, { useState } from 'react';

export default function RefundManager({ merchantAddress }) {
  const [orderId, setOrderId] = useState('');
  const [amount, setAmount] = useState('');
  const [reason, setReason] = useState('');
  const [status, setStatus] = useState(null);

  const handleSubmit = async (e) => {
    e.preventDefault();
    setStatus('Refund request submitted (connect contract invocation to process).');
    setOrderId(''); setAmount(''); setReason('');
  };

  return (
    <section aria-labelledby="refund-heading">
      <h2 id="refund-heading">Refund Management</h2>
      <form onSubmit={handleSubmit} aria-label="Initiate refund">
        <div>
          <label htmlFor="orderId">Order ID</label>
          <input id="orderId" value={orderId} onChange={e => setOrderId(e.target.value)} required />
        </div>
        <div>
          <label htmlFor="amount">Amount</label>
          <input id="amount" type="number" min="1" value={amount} onChange={e => setAmount(e.target.value)} required />
        </div>
        <div>
          <label htmlFor="reason">Reason</label>
          <input id="reason" value={reason} onChange={e => setReason(e.target.value)} required />
        </div>
        <button type="submit">Initiate Refund</button>
      </form>
      {status && <p role="status" aria-live="polite">{status}</p>}
    </section>
  );
}
