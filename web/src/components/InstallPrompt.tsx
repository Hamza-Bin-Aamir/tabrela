import { usePWAInstall } from '../hooks/usePWAInstall'
import { useState } from 'react'

/**
 * A subtle floating install button that appears on desktop only.
 * On mobile, the install option is in the hamburger menu.
 */
export default function InstallPrompt() {
  const { isInstallable, promptInstall } = usePWAInstall()
  const [dismissed, setDismissed] = useState(false)

  // Don't show if not installable or user dismissed it
  if (!isInstallable || dismissed) return null

  return (
    <div className="install-prompt">
      <div className="install-prompt-content">
        <div className="install-prompt-icon">
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
          </svg>
        </div>
        <div className="install-prompt-text">
          <span className="install-prompt-title">Install Tabrela</span>
          <span className="install-prompt-subtitle">Get the app experience</span>
        </div>
        <div className="install-prompt-actions">
          <button
            onClick={promptInstall}
            className="install-prompt-btn-install"
          >
            Install
          </button>
          <button
            onClick={() => setDismissed(true)}
            className="install-prompt-btn-dismiss"
            aria-label="Dismiss"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  )
}
