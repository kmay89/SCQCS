/**
 * SCQCS Site Scripts
 * Privacy-first security framework documentation
 * Optimized for runtime performance across browsers
 */

// ========================================
// Sound System - Subtle UI feedback
// Uses event delegation for efficiency
// ========================================
const SoundSystem = {
  ctx: null,
  enabled: true,
  volume: 0.025,

  init() {
    document.addEventListener('click', () => {
      if (!this.ctx) {
        this.ctx = new (window.AudioContext || window.webkitAudioContext)();
      }
    }, { once: true });
  },

  play(type) {
    if (!this.ctx || !this.enabled) return;

    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();

    osc.connect(gain);
    gain.connect(this.ctx.destination);

    const now = this.ctx.currentTime;

    switch(type) {
      case 'click':
        osc.frequency.setValueAtTime(800, now);
        osc.frequency.exponentialRampToValueAtTime(600, now + 0.05);
        gain.gain.setValueAtTime(this.volume, now);
        gain.gain.exponentialRampToValueAtTime(0.001, now + 0.08);
        osc.start(now);
        osc.stop(now + 0.08);
        break;

      case 'hover':
        osc.frequency.setValueAtTime(1200, now);
        osc.frequency.exponentialRampToValueAtTime(1000, now + 0.03);
        gain.gain.setValueAtTime(this.volume * 0.4, now);
        gain.gain.exponentialRampToValueAtTime(0.001, now + 0.05);
        osc.start(now);
        osc.stop(now + 0.05);
        break;

      case 'drawer':
        osc.type = 'sine';
        osc.frequency.setValueAtTime(400, now);
        osc.frequency.exponentialRampToValueAtTime(600, now + 0.1);
        gain.gain.setValueAtTime(this.volume, now);
        gain.gain.exponentialRampToValueAtTime(0.001, now + 0.15);
        osc.start(now);
        osc.stop(now + 0.15);
        break;
    }
  }
};

SoundSystem.init();

// ========================================
// Video Manager - Mobile memory optimization
// ========================================
const VideoManager = {
  video: null,
  observer: null,
  isVisible: true,
  // Use modern userAgentData API with fallback to regex
  isMobile: navigator.userAgentData?.mobile ?? /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent),
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
      this.video.pause();
      this.video.removeAttribute('autoplay');
      this.video.removeAttribute('loop');
      return;
    }

    this.setupVisibilityHandler();
    this.setupIntersectionObserver();
    this.setupErrorHandler();
    this.setupCleanup();

    // On mobile, be more aggressive about resource management
    if (this.isMobile) {
      this.setupMobileOptimizations();
    }
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
      if (this.isMobile && this.errorCount > 0) {
        // On mobile with previous errors, be cautious
        this.pause();
      }
    };
    this._addListener(this.video, 'waiting', waitingHandler);
  },

  setupMobileOptimizations() {
    // On mobile, control looping manually to save resources when not visible
    // Remove loop attribute to control it manually
    this.video.removeAttribute('loop');

    const endedHandler = () => {
      if (!this.isVisible || document.hidden) {
        return; // Don't loop if not visible - this is the main memory saver
      }

      // Just restart the video without reloading - avoids flash/page jump
      this.video.currentTime = 0;
      this.play();
    };
    this._addListener(this.video, 'ended', endedHandler);
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
    this.video.pause();
    this.video.removeAttribute('autoplay');
    this.video.removeAttribute('loop');
    this.video.load(); // Reset to show poster

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
        // Autoplay was prevented - this is fine on mobile
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
// ========================================
const QuantumAnimation = {
  running: true,
  startTime: null,
  rafId: null,

  // Animation durations (in ms)
  hueCycleDuration: 30000,  // 30s for full hue rotation
  pulseDuration: 9000,      // 9s for intensity pulse
  pulseDelay: 6000,         // 6s delay before pulse starts

  init() {
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
