import React, { useState } from 'react';
import { useWallet } from './hooks/useWallet';
import PaymentHistory from './components/PaymentHistory';
import RefundManager from './components/RefundManager';
import MerchantProfile from './components/MerchantProfile';
import { ThemeToggle } from './theme/ThemeToggle';
import './App.css';

const TABS = ['Payments', 'Refunds', 'Profile'];

export default function App() {
  const { publicKey, connect, disconnect, error } = useWallet();
  const [tab, setTab] = useState('Payments');

  return (
    <div className="app">
      <header className="app-header">
        <h1>Pulsar Merchant Dashboard</h1>
        <div className="app-header-actions">
          <ThemeToggle />
          {publicKey ? (
            <div className="wallet-info">
              <span title={publicKey}>{publicKey.slice(0, 8)}…{publicKey.slice(-4)}</span>
              <button onClick={disconnect}>Disconnect</button>
            </div>
          ) : (
            <button onClick={connect}>Connect Freighter</button>
          )}
        </div>
      </header>

      {error && <p role="alert" className="error">{error}</p>}

      {!publicKey ? (
        <main className="connect-prompt">
          <p>Connect your Freighter wallet to access the dashboard.</p>
        </main>
      ) : (
        <main>
          <nav aria-label="Dashboard sections">
            <ul role="tablist" className="tabs">
              {TABS.map(t => (
                <li key={t} role="presentation">
                  <button
                    role="tab"
                    aria-selected={tab === t}
                    onClick={() => setTab(t)}
                  >
                    {t}
                  </button>
                </li>
              ))}
            </ul>
          </nav>

          <div role="tabpanel" aria-label={tab} className="tab-content">
            {tab === 'Payments' && <PaymentHistory merchantAddress={publicKey} />}
            {tab === 'Refunds'  && <RefundManager  merchantAddress={publicKey} />}
            {tab === 'Profile'  && <MerchantProfile publicKey={publicKey} />}
          </div>
        </main>
      )}
    </div>
  );
}
