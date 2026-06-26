import { useState, useEffect, useRef } from 'react';
import './SettingsPanel.css';

const DEFAULTS = {
  display: { theme: 'light', language: 'en', dateFormat: 'MM/DD/YYYY' },
  notifications: { emailAlerts: true, paymentSuccess: true, paymentFailure: true },
  security: { twoFactor: false, sessionTimeout: '30' },
};

export function SettingsPanel({ initialSettings, onSave }) {
  const [settings, setSettings] = useState(initialSettings || DEFAULTS);
  const [showConfirm, setShowConfirm] = useState(false);
  const [toast, setToast] = useState(null);
  const toastTimer = useRef(null);

  useEffect(() => () => clearTimeout(toastTimer.current), []);

  function showToast(msg, type = 'success') {
    setToast({ msg, type });
    clearTimeout(toastTimer.current);
    toastTimer.current = setTimeout(() => setToast(null), 3000);
  }

  function set(section, key, value) {
    setSettings(s => ({ ...s, [section]: { ...s[section], [key]: value } }));
  }

  function handleSave(e) {
    e.preventDefault();
    onSave?.(settings);
    showToast('Settings saved successfully.');
  }

  function handleReset() {
    setSettings(DEFAULTS);
    setShowConfirm(false);
    showToast('Settings reset to defaults.', 'info');
  }

  const { display, notifications, security } = settings;

  return (
    <div className="sp-container" role="main" aria-label="Settings Panel">
      {toast && (
        <div className={`sp-toast sp-toast--${toast.type}`} role="alert" aria-live="polite">
          {toast.msg}
        </div>
      )}

      <form onSubmit={handleSave} className="sp-form" noValidate>
        {/* Display */}
        <fieldset className="sp-section">
          <legend className="sp-section__title">Display</legend>
          <div className="sp-field">
            <label htmlFor="sp-theme">Theme</label>
            <select
              id="sp-theme"
              value={display.theme}
              onChange={e => set('display', 'theme', e.target.value)}
            >
              <option value="light">Light</option>
              <option value="dark">Dark</option>
              <option value="system">System</option>
            </select>
          </div>
          <div className="sp-field">
            <label htmlFor="sp-language">Language</label>
            <select
              id="sp-language"
              value={display.language}
              onChange={e => set('display', 'language', e.target.value)}
            >
              <option value="en">English</option>
              <option value="fr">French</option>
              <option value="es">Spanish</option>
              <option value="de">German</option>
            </select>
          </div>
          <div className="sp-field">
            <label htmlFor="sp-dateformat">Date Format</label>
            <select
              id="sp-dateformat"
              value={display.dateFormat}
              onChange={e => set('display', 'dateFormat', e.target.value)}
            >
              <option value="MM/DD/YYYY">MM/DD/YYYY</option>
              <option value="DD/MM/YYYY">DD/MM/YYYY</option>
              <option value="YYYY-MM-DD">YYYY-MM-DD</option>
            </select>
          </div>
        </fieldset>

        {/* Notifications */}
        <fieldset className="sp-section">
          <legend className="sp-section__title">Notifications</legend>
          {[
            ['emailAlerts', 'Email Alerts'],
            ['paymentSuccess', 'Payment Success'],
            ['paymentFailure', 'Payment Failure'],
          ].map(([key, label]) => (
            <div className="sp-field sp-field--toggle" key={key}>
              <label htmlFor={`sp-${key}`}>{label}</label>
              <input
                id={`sp-${key}`}
                type="checkbox"
                role="switch"
                aria-checked={notifications[key]}
                checked={notifications[key]}
                onChange={e => set('notifications', key, e.target.checked)}
              />
            </div>
          ))}
        </fieldset>

        {/* Security */}
        <fieldset className="sp-section">
          <legend className="sp-section__title">Security</legend>
          <div className="sp-field sp-field--toggle">
            <label htmlFor="sp-2fa">Two-Factor Authentication</label>
            <input
              id="sp-2fa"
              type="checkbox"
              role="switch"
              aria-checked={security.twoFactor}
              checked={security.twoFactor}
              onChange={e => set('security', 'twoFactor', e.target.checked)}
            />
          </div>
          <div className="sp-field">
            <label htmlFor="sp-session">Session Timeout</label>
            <select
              id="sp-session"
              value={security.sessionTimeout}
              onChange={e => set('security', 'sessionTimeout', e.target.value)}
            >
              <option value="15">15 minutes</option>
              <option value="30">30 minutes</option>
              <option value="60">1 hour</option>
              <option value="240">4 hours</option>
            </select>
          </div>
        </fieldset>

        <div className="sp-actions">
          <button type="button" className="sp-btn sp-btn--ghost" onClick={() => setShowConfirm(true)}>
            Reset to Defaults
          </button>
          <button type="submit" className="sp-btn sp-btn--primary">
            Save Changes
          </button>
        </div>
      </form>

      {/* Confirmation dialog for destructive reset */}
      {showConfirm && (
        <div className="sp-overlay" role="dialog" aria-modal="true" aria-labelledby="sp-confirm-title">
          <div className="sp-dialog">
            <h2 id="sp-confirm-title" className="sp-dialog__title">Reset Settings?</h2>
            <p className="sp-dialog__body">
              This will reset all settings to their default values. This action cannot be undone.
            </p>
            <div className="sp-dialog__actions">
              <button className="sp-btn sp-btn--ghost" onClick={() => setShowConfirm(false)}>
                Cancel
              </button>
              <button className="sp-btn sp-btn--danger" onClick={handleReset} autoFocus>
                Reset
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
