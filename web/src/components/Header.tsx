import { Link } from 'react-router-dom'
import { useState } from 'react'

export default function Header() {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false)

  const toggleMenu = () => setMobileMenuOpen(!mobileMenuOpen)
  const closeMenu = () => setMobileMenuOpen(false)

  return (
    <header className="site-header" role="banner">
      <div className="container mx-auto px-6 flex items-center justify-between max-w-[1400px] h-full">
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg flex items-center justify-center">
            <span className="text-white font-bold text-lg">T</span>
          </div>
          <h1 className="text-2xl md:text-3xl font-extrabold tracking-tight text-white">
            Tabrela
          </h1>
        </div>

        {/* Hamburger button - visible on mobile only */}
        <button
          className="hamburger-btn"
          onClick={toggleMenu}
          aria-label="Toggle menu"
          aria-expanded={mobileMenuOpen}
        >
          <span className="hamburger-line"></span>
          <span className="hamburger-line"></span>
          <span className="hamburger-line"></span>
        </button>

        {/* Desktop navigation - hidden on mobile */}
        <nav className="desktop-nav flex items-center gap-3" aria-label="Main navigation">
          <Link 
            to="/" 
            className="nav-link"
            style={{
              padding: '0.5rem 1rem',
              fontSize: '0.875rem',
              fontWeight: '500',
              color: '#d1d5db',
              borderRadius: '0.5rem',
              transition: 'all 0.2s',
              textDecoration: 'none'
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.color = '#ffffff';
              e.currentTarget.style.backgroundColor = 'rgba(255, 255, 255, 0.1)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.color = '#d1d5db';
              e.currentTarget.style.backgroundColor = 'transparent';
            }}
          >
            Home
          </Link>
          <Link 
            to="/loading" 
            className="nav-link"
            style={{
              padding: '0.5rem 1rem',
              fontSize: '0.875rem',
              fontWeight: '500',
              color: '#d1d5db',
              borderRadius: '0.5rem',
              transition: 'all 0.2s',
              textDecoration: 'none'
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.color = '#ffffff';
              e.currentTarget.style.backgroundColor = 'rgba(255, 255, 255, 0.1)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.color = '#d1d5db';
              e.currentTarget.style.backgroundColor = 'transparent';
            }}
          >
            Loading
          </Link>
          <Link 
            to="/error" 
            className="nav-link"
            style={{
              padding: '0.5rem 1rem',
              fontSize: '0.875rem',
              fontWeight: '500',
              color: '#d1d5db',
              borderRadius: '0.5rem',
              transition: 'all 0.2s',
              textDecoration: 'none'
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.color = '#ffffff';
              e.currentTarget.style.backgroundColor = 'rgba(255, 255, 255, 0.1)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.color = '#d1d5db';
              e.currentTarget.style.backgroundColor = 'transparent';
            }}
          >
            Error
          </Link>
        </nav>

        {/* Mobile navigation - slide-in menu */}
        <nav 
          className={`mobile-nav ${mobileMenuOpen ? 'mobile-nav-open' : ''}`}
          aria-label="Mobile navigation"
        >
          <Link 
            to="/" 
            className="mobile-nav-link"
            onClick={closeMenu}
          >
            Home
          </Link>
          <Link 
            to="/loading" 
            className="mobile-nav-link"
            onClick={closeMenu}
          >
            Loading
          </Link>
          <Link 
            to="/error" 
            className="mobile-nav-link"
            onClick={closeMenu}
          >
            Error
          </Link>
        </nav>

        {/* Backdrop overlay for mobile menu */}
        {mobileMenuOpen && (
          <div 
            className="mobile-nav-backdrop"
            onClick={closeMenu}
            aria-hidden="true"
          />
        )}
      </div>
      {/* Fun accent bar */}
      <div className="absolute bottom-0 left-0 right-0 h-1 bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500"></div>
    </header>
  )
}