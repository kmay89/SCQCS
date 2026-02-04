/**
 * SCQCS Site Scripts
 * Privacy-first security framework documentation
 */

// ========================================
// Sound System - Subtle UI feedback
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
// Attach sound events
// ========================================
document.querySelectorAll('[data-sound="click"]').forEach(el => {
  el.addEventListener('click', () => SoundSystem.play('click'));
});

document.querySelectorAll('[data-sound="hover"]').forEach(el => {
  el.addEventListener('mouseenter', () => SoundSystem.play('hover'));
});

document.querySelectorAll('[data-sound="drawer"]').forEach(el => {
  el.addEventListener('toggle', () => SoundSystem.play('drawer'));
});

// ========================================
// Set year
// ========================================
const year = new Date().getFullYear();
document.getElementById('year').textContent = year;
document.getElementById('footer-year').textContent = year;

// ========================================
// Navigation scroll effect
// ========================================
const nav = document.getElementById('nav');
const scrollProgress = document.getElementById('scroll-progress');

window.addEventListener('scroll', () => {
  // Nav background
  if (window.pageYOffset > 50) {
    nav.classList.add('scrolled');
  } else {
    nav.classList.remove('scrolled');
  }

  // Scroll progress bar
  const scrollHeight = document.documentElement.scrollHeight - window.innerHeight;
  const scrollPercent = (window.pageYOffset / scrollHeight) * 100;
  scrollProgress.style.width = scrollPercent + '%';

  // Shift gradient position based on scroll for color transition
  scrollProgress.style.backgroundPosition = scrollPercent + '% 0';
}, { passive: true });

// ========================================
// Scroll reveal animation
// ========================================
const observerOptions = {
  root: null,
  rootMargin: '0px 0px -100px 0px',
  threshold: 0.1
};

const observer = new IntersectionObserver((entries) => {
  entries.forEach(entry => {
    if (entry.isIntersecting) {
      entry.target.classList.add('visible');
    }
  });
}, observerOptions);

document.querySelectorAll('.reveal').forEach(el => {
  observer.observe(el);
});

// ========================================
// Smooth scroll for anchor links
// ========================================
document.querySelectorAll('a[href^="#"]').forEach(anchor => {
  anchor.addEventListener('click', function (e) {
    e.preventDefault();
    const target = document.querySelector(this.getAttribute('href'));
    if (target) {
      target.scrollIntoView({
        behavior: 'smooth',
        block: 'start'
      });
    }
  });
});

// ========================================
// Scroll indicator click
// ========================================
document.querySelector('.scroll-indicator').addEventListener('click', () => {
  document.querySelector('#what').scrollIntoView({ behavior: 'smooth' });
});
