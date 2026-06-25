import { useState, useCallback, useEffect } from 'react';
import './SettingsPanel.css';

const DEFAULTS = {
  theme: 'light',
  language: 'en',
  dateFormat: 'YYYY-MM-DD',
  emailAlerts: true,
  paymentSuccess: true,
  paymentFailure: true,
  twoFA: false,
  sessionTimeout: '30',
};

export default function SettingsPanel({ onSave }) {
  const [settings, setSettings] = useState(DEFAULTS);
  const [showConfirm, setShowConfirm] = useState(false);
  const [toast, setToast] = useState(null);

  const set = useCallback((key, value) => {
    setSettings(prev => ({ ...prev, [key]: value }));
  }, []);

  const handleSave = async () => {
    await onSave?.(settings);
    setToast('Settings saved.');
  };

  const handleReset = () => setShowConfirm(true);

  const confirmReset = () => {
    setSettings(DEFAULTS);
    setShowConfirm(false);
    setToast('Settings reset to defaults.');
  };

  useEffect(() => {
    if (!toast) return;
    const id = setTimeout(() => setToast(null), 3000);
    return () => clearTimeout(id);
  }, [toast]);

  return (
    <div className="sp-container">
      <h2 className="sp-title">Settings</h2>

      {/* Display */}
      <fieldset className="sp-section">
        <legend>Display</legend>

        <label htmlFor="sp-theme">Theme</label>
        <select id="sp-theme" value={settings.theme} onChange={e => set('theme', e.target.value)}>
          <option value="light">Light</option>
          <option value="dark">Dark</option>
        </select>

        <label htmlFor="sp-language">Language</label>
        <select id="sp-language" value={settings.language} onChange={e => set('language', e.target.value)}>
          <option value="en">English</option>
          <option value="es">Español</option>
          <option value="fr">Français</option>
          <option value="de">Deutsch</option>
        </select>

        <label htmlFor="sp-date-format">Date Format</label>
        <select id="sp-date-format" value={settings.dateFormat} onChange={e => set('dateFormat', e.target.value)}>
          <option value="YYYY-MM-DD">YYYY-MM-DD</option>
          <option value="MM/DD/YYYY">MM/DD/YYYY</option>
          <option value="DD/MM/YYYY">DD/MM/YYYY</option>
        </select>
      </fieldset>

      {/* Notifications */}
      <fieldset className="sp-section">
        <legend>Notifications</legend>

        <label className="sp-toggle">
          <input type="checkbox" checked={settings.emailAlerts} onChange={e => set('emailAlerts', e.target.checked)} />
          Email Alerts
        </label>

        <label className="sp-toggle">
          <input type="checkbox" checked={settings.paymentSuccess} onChange={e => set('paymentSuccess', e.target.checked)} />
          Payment Success
        </label>

        <label className="sp-toggle">
          <input type="checkbox" checked={settings.paymentFailure} onChange={e => set('paymentFailure', e.target.checked)} />
          Payment Failure
        </label>
      </fieldset>

      {/* Security */}
      <fieldset className="sp-section">
        <legend>Security</legend>

        <label className="sp-toggle">
          <input type="checkbox" checked={settings.twoFA} onChange={e => set('twoFA', e.target.checked)} />
          Two-Factor Authentication (2FA)
        </label>

        <label htmlFor="sp-session-timeout">Session Timeout (minutes)</label>
        <select id="sp-session-timeout" value={settings.sessionTimeout} onChange={e => set('sessionTimeout', e.target.value)}>
          <option value="15">15</option>
          <option value="30">30</option>
          <option value="60">60</option>
          <option value="120">120</option>
        </select>
      </fieldset>

      <div className="sp-actions">
        <button className="sp-btn-secondary" onClick={handleReset}>Reset to Defaults</button>
        <button className="sp-btn-primary" onClick={handleSave}>Save</button>
      </div>

      {/* Confirmation dialog */}
      {showConfirm && (
        <div className="sp-overlay" role="dialog" aria-modal="true" aria-labelledby="sp-confirm-title">
          <div className="sp-dialog">
            <h3 id="sp-confirm-title">Reset to Defaults?</h3>
            <p>All settings will be restored to their default values. This cannot be undone.</p>
            <div className="sp-dialog-actions">
              <button className="sp-btn-secondary" onClick={() => setShowConfirm(false)}>Cancel</button>
              <button className="sp-btn-danger" onClick={confirmReset}>Reset</button>
            </div>
          </div>
        </div>
      )}

      {/* Toast */}
      {toast && (
        <div className="sp-toast" role="status" aria-live="polite">{toast}</div>
      )}
    </div>
  );
}
