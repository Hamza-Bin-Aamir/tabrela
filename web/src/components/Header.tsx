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
          <img 
            src="/logos/tabrela.png" 
            alt="Tabrela Logo" 
            className="header-logo"
          />
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
          <Link to="/" className="nav-link">
            Home
          </Link>
          <Link to="/about" className="nav-link">
            About
          </Link>
          <Link to="/login" className="nav-link">
            Log in
          </Link>
          <Link to="/signup" className="nav-link">
            Sign up
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
            to="/about" 
            className="mobile-nav-link"
            onClick={closeMenu}
          >
            About
          </Link>
          <Link 
            to="/login" 
            className="mobile-nav-link"
            onClick={closeMenu}
          >
            Log in
          </Link>
          <Link 
            to="/signup" 
            className="mobile-nav-link"
            onClick={closeMenu}
          >
            Sign up
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