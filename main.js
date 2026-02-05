/**
 * SCQCS Site Scripts
 * Privacy-first security framework documentation
 * Optimized for runtime performance across browsers
 */

// ========================================
// Sound System - Friendly haptic UI feedback
// Warm, cute, satisfying sounds like pressing a quality button
// Uses event delegation for efficiency
// ========================================
const SoundSystem = {
  ctx: null,
  enabled: true,
  volume: 0.035,

  play(type) {
    if (!this.enabled) return;

    // Lazily create AudioContext on first sound (requires user gesture)
    if (!this.ctx) {
      this.ctx = new (window.AudioContext || window.webkitAudioContext)();
    }

    // Helper to create connected oscillator + gain node pair
    const createVoice = () => {
      const osc = this.ctx.createOscillator();
      const gain = this.ctx.createGain();
      osc.connect(gain);
      gain.connect(this.ctx.destination);
      return { osc, gain };
    };

    const now = this.ctx.currentTime;

    switch(type) {
      case 'click': {
        // Warm, satisfying haptic pop - like pressing a quality mechanical button
        // Two layered tones create richness: fundamental + soft harmonic
        const { osc: osc1, gain: gain1 } = createVoice();
        const { osc: osc2, gain: gain2 } = createVoice();

        // Primary tone - warm pop with satisfying pitch drop
        osc1.type = 'sine';
        osc1.frequency.setValueAtTime(520, now);
        osc1.frequency.exponentialRampToValueAtTime(340, now + 0.06);
        gain1.gain.setValueAtTime(this.volume, now);
        gain1.gain.exponentialRampToValueAtTime(0.001, now + 0.07);

        // Harmonic layer - adds "click" presence without harshness
        osc2.type = 'triangle';
        osc2.frequency.setValueAtTime(880, now);
        osc2.frequency.exponentialRampToValueAtTime(660, now + 0.04);
        gain2.gain.setValueAtTime(this.volume * 0.25, now);
        gain2.gain.exponentialRampToValueAtTime(0.001, now + 0.05);

        osc1.start(now);
        osc2.start(now);
        osc1.stop(now + 0.07);
        osc2.stop(now + 0.05);
        break;
      }

      case 'hover': {
        // Soft, gentle brush - barely there but pleasant
        const { osc, gain } = createVoice();

        osc.type = 'sine';
        osc.frequency.setValueAtTime(680, now);
        osc.frequency.exponentialRampToValueAtTime(580, now + 0.035);
        gain.gain.setValueAtTime(this.volume * 0.2, now);
        gain.gain.exponentialRampToValueAtTime(0.001, now + 0.04);

        osc.start(now);
        osc.stop(now + 0.04);
        break;
      }

      case 'drawer': {
        // Gentle reveal chime - warm and welcoming
        // Two notes in harmony for a pleasant "opening" feel
        const { osc: osc1, gain: gain1 } = createVoice();
        const { osc: osc2, gain: gain2 } = createVoice();

        // Base tone - warm ascending note
        osc1.type = 'sine';
        osc1.frequency.setValueAtTime(380, now);
        osc1.frequency.exponentialRampToValueAtTime(480, now + 0.12);
        gain1.gain.setValueAtTime(this.volume * 0.8, now);
        gain1.gain.exponentialRampToValueAtTime(0.001, now + 0.15);

        // Harmony - soft fifth above, slightly delayed
        osc2.type = 'sine';
        osc2.frequency.setValueAtTime(570, now + 0.02);
        osc2.frequency.exponentialRampToValueAtTime(720, now + 0.14);
        gain2.gain.setValueAtTime(0, now);
        gain2.gain.setValueAtTime(this.volume * 0.4, now + 0.02);
        gain2.gain.exponentialRampToValueAtTime(0.001, now + 0.16);

        osc1.start(now);
        osc2.start(now);
        osc1.stop(now + 0.15);
        osc2.stop(now + 0.16);
        break;
      }
    }
  }
};

// ========================================
// Video Manager - Visibility and error handling
// ========================================
const VideoManager = {
  video: null,
  observer: null,
  isVisible: true,
  prefersReducedMotion: window.matchMedia('(prefers-reduced-motion: reduce)').matches,
  errorCount: 0,
  maxErrors: 3,
  // Track all event listeners for automatic cleanup
  _listeners: [],

  // Helper to add and track event listeners
  _addListener(target, type, handler, options) {
    target.addEventListener(type, handler, options);
    this._listeners.push({ target, type, handler, options });
  },

  init() {
    this.video = document.querySelector('.hero-video');
    if (!this.video) return;

    // If user prefers reduced motion, show poster only
    if (this.prefersReducedMotion) {
      this.replaceWithPoster();
      return;
    }

    // Video now works on mobile - the refresh/scroll bug was caused by
    // the quantum overlay (body::before), not the video element
    this.setupVisibilityHandler();
    this.setupIntersectionObserver();
    this.setupErrorHandler();
    this.setupCleanup();
  },

  // Replace video element with a static image to completely free video memory
  replaceWithPoster() {
    if (!this.video) return;

    const poster = this.video.getAttribute('poster');
    if (!poster) return;

    // Create img element with same styling
    const img = document.createElement('img');
    img.src = poster;
    img.alt = 'SCQCS Hero';
    img.className = 'hero-video'; // Reuse same class for styling
    img.setAttribute('loading', 'eager');
    img.setAttribute('decoding', 'async');

    // Replace video with img
    this.video.parentNode.replaceChild(img, this.video);
    this.video = null;
  },

  setupVisibilityHandler() {
    const handler = () => {
      if (document.hidden) {
        this.pause();
      } else if (this.isVisible) {
        this.play();
      }
    };
    this._addListener(document, 'visibilitychange', handler);
  },

  setupIntersectionObserver() {
    // Pause video when scrolled out of view
    this.observer = new IntersectionObserver((entries) => {
      entries.forEach(entry => {
        this.isVisible = entry.isIntersecting;
        if (entry.isIntersecting && !document.hidden) {
          this.play();
        } else {
          this.pause();
        }
      });
    }, {
      threshold: 0.1,
      rootMargin: '50px'
    });

    this.observer.observe(this.video);
  },

  setupErrorHandler() {
    // Handle video errors gracefully
    const errorHandler = (e) => {
      this.errorCount++;
      console.warn('Video error:', e);

      if (this.errorCount >= this.maxErrors) {
        this.disable();
      }
    };
    this._addListener(this.video, 'error', errorHandler);

    // Handle stall/waiting events that might indicate memory issues
    const waitingHandler = () => {
      if (this.errorCount > 0) {
        // With previous errors, be cautious
        this.pause();
      }
    };
    this._addListener(this.video, 'waiting', waitingHandler);
  },

  setupCleanup() {
    // Pause video when leaving page, but don't clear src (causes issues with bfcache)
    this._addListener(window, 'pagehide', () => this.pause());
    this._addListener(window, 'beforeunload', () => this.pause());

    // Handle page restore from bfcache
    const pageshowHandler = (event) => {
      if (event.persisted && this.isVisible && !document.hidden) {
        this.play();
      }
    };
    this._addListener(window, 'pageshow', pageshowHandler);
  },

  // Permanently disable video and clean up all listeners
  disable() {
    // Replace with poster instead of calling load() which can cause scroll issues
    this.replaceWithPoster();

    // Disconnect observer
    if (this.observer) {
      this.observer.disconnect();
      this.observer = null;
    }

    // Remove all tracked event listeners
    this._listeners.forEach(({ target, type, handler, options }) => {
      target.removeEventListener(type, handler, options);
    });
    this._listeners = [];
  },

  play() {
    if (!this.video || this.prefersReducedMotion || this.errorCount >= this.maxErrors) return;

    const playPromise = this.video.play();
    if (playPromise !== undefined) {
      playPromise.catch(err => {
        // Autoplay was prevented - this is fine
        console.log('Video autoplay prevented:', err.message);
      });
    }
  },

  pause() {
    if (!this.video) return;
    this.video.pause();
  }
};

// Initialize after DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => VideoManager.init());
} else {
  VideoManager.init();
}

// ========================================
// Event Delegation - Single listeners for better performance
// Replaces multiple querySelectorAll().forEach() patterns
// ========================================

// Delegated sound events - single listener per event type instead of many
document.addEventListener('click', (e) => {
  const target = e.target.closest('[data-sound="click"]');
  if (target) SoundSystem.play('click');
}, { passive: true });

document.addEventListener('mouseenter', (e) => {
  const target = e.target.closest('[data-sound="hover"]');
  if (target) SoundSystem.play('hover');
}, { capture: true, passive: true });

document.addEventListener('toggle', (e) => {
  if (e.target.matches('[data-sound="drawer"]')) {
    SoundSystem.play('drawer');
  }
}, { capture: true, passive: true });

// ========================================
// Set year - cached DOM references
// ========================================
const yearEl = document.getElementById('year');
const footerYearEl = document.getElementById('footer-year');
const year = new Date().getFullYear();
if (yearEl) yearEl.textContent = year;
if (footerYearEl) footerYearEl.textContent = year;

// ========================================
// Navigation scroll effect (optimized with rAF)
// Removed backgroundPosition for better performance
// ========================================
const nav = document.getElementById('nav');
const scrollProgress = document.getElementById('scroll-progress');

let ticking = false;
let lastScrollY = 0;
// Cache scroll height calculation - recalculate only on resize
let cachedScrollHeight = document.documentElement.scrollHeight - window.innerHeight;

function updateScroll() {
  const scrollY = lastScrollY;

  // Nav background
  if (scrollY > 50) {
    nav.classList.add('scrolled');
  } else {
    nav.classList.remove('scrolled');
  }

  // Scroll progress bar - using transform only for GPU acceleration
  // Removed backgroundPosition to avoid repaints
  const scrollPercent = cachedScrollHeight > 0 ? scrollY / cachedScrollHeight : 0;
  scrollProgress.style.transform = `scaleX(${scrollPercent})`;

  ticking = false;
}

window.addEventListener('scroll', () => {
  lastScrollY = window.scrollY;

  if (!ticking) {
    requestAnimationFrame(updateScroll);
    ticking = true;
  }
}, { passive: true });

// Update cached scroll height on resize (debounced)
let resizeTimeout;
window.addEventListener('resize', () => {
  clearTimeout(resizeTimeout);
  resizeTimeout = setTimeout(() => {
    cachedScrollHeight = document.documentElement.scrollHeight - window.innerHeight;
  }, 150);
}, { passive: true });

// ========================================
// Scroll reveal animation - optimized observer
// ========================================
const observerOptions = {
  root: null,
  rootMargin: '0px 0px -100px 0px',
  threshold: 0.1
};

const revealObserver = new IntersectionObserver((entries) => {
  entries.forEach(entry => {
    if (entry.isIntersecting) {
      entry.target.classList.add('visible');
      // Unobserve after reveal to free resources
      revealObserver.unobserve(entry.target);
    }
  });
}, observerOptions);

document.querySelectorAll('.reveal').forEach(el => {
  revealObserver.observe(el);
});

// ========================================
// Smooth scroll - event delegation for anchor links
// ========================================
document.addEventListener('click', (e) => {
  const anchor = e.target.closest('a[href^="#"]');
  if (!anchor) return;

  const href = anchor.getAttribute('href');
  if (!href || href === '#') return;

  const target = document.querySelector(href);
  if (target) {
    e.preventDefault();
    target.scrollIntoView({
      behavior: 'smooth',
      block: 'start'
    });
  }
});

// ========================================
// Scroll indicator click
// ========================================
const scrollIndicator = document.querySelector('.scroll-indicator');
if (scrollIndicator) {
  scrollIndicator.addEventListener('click', () => {
    const target = document.querySelector('#what');
    if (target) {
      target.scrollIntoView({ behavior: 'smooth' });
    }
  }, { passive: true });
}

// ========================================
// Mobile Navigation Toggle
// ========================================
const MobileNav = {
  hamburger: null,
  mobileNav: null,
  isOpen: false,

  init() {
    this.hamburger = document.getElementById('hamburger');
    this.mobileNav = document.getElementById('mobile-nav');

    if (!this.hamburger || !this.mobileNav) return;

    // Toggle menu on hamburger click/touch
    // Use both click and touchend for better iOS support
    const handleToggle = (e) => {
      e.preventDefault();
      e.stopPropagation();
      this.toggle();
    };

    this.hamburger.addEventListener('click', handleToggle);
    // Touchend fires more reliably on iOS Safari
    this.hamburger.addEventListener('touchend', handleToggle, { passive: false });

    // Close menu when clicking/touching a link
    // Use shared idempotent handler to prevent double-firing on touch devices
    this.mobileNav.querySelectorAll('a').forEach(link => {
      const handleClose = () => {
        if (this.isOpen) {
          this.close();
        }
      };
      link.addEventListener('click', handleClose);
      link.addEventListener('touchend', handleClose, { passive: true });
    });

    // Close menu on escape key
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && this.isOpen) {
        this.close();
      }
    });

    // Close menu when clicking/touching outside
    // Use shared idempotent handler to prevent double-firing on touch devices
    const handleOutsideClose = (e) => {
      if (e.target === this.mobileNav && this.isOpen) {
        this.close();
      }
    };
    this.mobileNav.addEventListener('click', handleOutsideClose);
    this.mobileNav.addEventListener('touchend', handleOutsideClose, { passive: true });
  },

  toggle() {
    if (this.isOpen) {
      this.close();
    } else {
      this.open();
    }
  },

  open() {
    this.isOpen = true;
    this.hamburger.classList.add('active');
    this.hamburger.setAttribute('aria-expanded', 'true');
    this.mobileNav.classList.add('active');
    document.body.style.overflow = 'hidden';
    SoundSystem.play('drawer');
  },

  close() {
    this.isOpen = false;
    this.hamburger.classList.remove('active');
    this.hamburger.setAttribute('aria-expanded', 'false');
    this.mobileNav.classList.remove('active');
    document.body.style.overflow = '';
    SoundSystem.play('click');
  }
};

// Initialize mobile navigation
MobileNav.init();

// ========================================
// Copy prompt to clipboard - optimized
// ========================================

// Cache LLM items once at load time
const llmItems = document.querySelectorAll('.ai-prompt-llm-item');
const animationDuration = 8000; // 8 seconds

// Keyframes matching the CSS animation `rotate-llm`
const keyframes = [
  { start: 0.75, index: 3 }, // 75-95%
  { start: 0.50, index: 2 }, // 50-70%
  { start: 0.25, index: 1 }, // 25-45%
  { start: 0.00, index: 0 }  // 0-20%
];

// Reusable textarea for clipboard fallback (lazy created)
let fallbackTextarea = null;

function copyPrompt(button) {
  // Get the currently visible LLM name based on animation timing
  const animationProgress = (Date.now() % animationDuration) / animationDuration;

  const currentKeyframe = keyframes.find(kf => animationProgress >= kf.start);
  const currentIndex = currentKeyframe ? currentKeyframe.index : 0;

  const currentLLM = llmItems[currentIndex]?.textContent || 'Claude';

  // Build the full prompt
  const prompt = `${currentLLM}, use github.com/kmay89/SCQCS to build me a site about `;

  const showCopied = () => {
    button.classList.add('copied');
    setTimeout(() => {
      button.classList.remove('copied');
    }, 2000);
  };

  navigator.clipboard.writeText(prompt).then(showCopied).catch(() => {
    // Fallback for older browsers - reuse textarea
    if (!fallbackTextarea) {
      fallbackTextarea = document.createElement('textarea');
      fallbackTextarea.style.cssText = 'position:fixed;opacity:0;pointer-events:none;';
      fallbackTextarea.setAttribute('aria-hidden', 'true');
      document.body.appendChild(fallbackTextarea);
    }

    fallbackTextarea.value = prompt;
    fallbackTextarea.select();
    if (document.execCommand('copy')) {
      showCopied();
    }
  });
}

// Event delegation for copy buttons
document.addEventListener('click', (e) => {
  const button = e.target.closest('.ai-prompt-copy');
  if (button) copyPrompt(button);
});

// ========================================
// Quantum Animation System
// JavaScript-driven to avoid browser rendering issues
// with @property CSS animations on the html element.
// Safari/Firefox can flash white when CSS animations cycle.
// DISABLED ON MOBILE: The overlay is hidden on mobile via CSS,
// so we skip the animation entirely to save CPU/battery.
// ========================================
const QuantumAnimation = {
  running: true,
  startTime: null,
  rafId: null,
  // Use matchMedia to align with CSS and correctly detect when to disable animation
  isMobile: window.matchMedia('(max-width: 768px), (hover: none) and (pointer: coarse)').matches,

  // Animation durations (in ms)
  hueCycleDuration: 30000,  // 30s for full hue rotation
  pulseDuration: 9000,      // 9s for intensity pulse
  pulseDelay: 6000,         // 6s delay before pulse starts

  init() {
    // Skip animation on mobile - the overlay is hidden via CSS anyway
    // This saves CPU/battery on mobile devices
    if (this.isMobile) {
      return;
    }

    this.startTime = performance.now();
    this.animate();

    // Pause when page not visible (saves battery)
    document.addEventListener('visibilitychange', () => {
      if (document.hidden) {
        this.pause();
      } else {
        this.resume();
      }
    });
  },

  animate() {
    if (!this.running) return;

    const now = performance.now();
    const elapsed = now - this.startTime;

    // Hue cycles continuously from 0 to 360 (30s cycle)
    // Using modulo avoids the discontinuity that causes browser repaints
    const hue = (elapsed / this.hueCycleDuration * 360) % 360;

    // Intensity pulses with easeInOutSine (9s cycle, 6s delay)
    let intensity = 0;
    const pulseElapsed = elapsed - this.pulseDelay;
    if (pulseElapsed > 0) {
      const pulseProgress = (pulseElapsed % this.pulseDuration) / this.pulseDuration;
      // easeInOutSine: peaks at 0.5, returns to 0 at 1.0
      intensity = Math.sin(pulseProgress * Math.PI);
    }

    // Apply to document element
    const root = document.documentElement;
    root.style.setProperty('--quantum-hue', `${hue}deg`);
    root.style.setProperty('--quantum-intensity', intensity.toFixed(3));

    this.rafId = requestAnimationFrame(() => this.animate());
  },

  pause() {
    this.running = false;
    if (this.rafId) {
      cancelAnimationFrame(this.rafId);
      this.rafId = null;
    }
  },

  resume() {
    if (!this.running) {
      this.running = true;
      // Adjust start time to maintain smooth animation continuity
      this.animate();
    }
  }
};

// Start quantum animations
QuantumAnimation.init();
