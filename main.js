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
// Navigation scroll effect (optimized with rAF)
// ========================================
const nav = document.getElementById('nav');
const scrollProgress = document.getElementById('scroll-progress');

let ticking = false;
let lastScrollY = 0;

function updateScroll() {
  const scrollY = lastScrollY;

  // Nav background
  if (scrollY > 50) {
    nav.classList.add('scrolled');
  } else {
    nav.classList.remove('scrolled');
  }

  // Scroll progress bar - using transform for GPU acceleration
  const scrollHeight = document.documentElement.scrollHeight - window.innerHeight;
  const scrollPercent = scrollHeight > 0 ? scrollY / scrollHeight : 0;

  // Use transform for instant, GPU-accelerated updates
  scrollProgress.style.transform = `scaleX(${scrollPercent})`;
  scrollProgress.style.backgroundPosition = `${scrollPercent * 100}% 0`;

  ticking = false;
}

window.addEventListener('scroll', () => {
  lastScrollY = window.scrollY;

  if (!ticking) {
    requestAnimationFrame(updateScroll);
    ticking = true;
  }
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

// ========================================
// Copy prompt to clipboard
// ========================================
function copyPrompt(button) {
  // Get the currently visible LLM name based on animation timing
  const llmItems = document.querySelectorAll('.ai-prompt-llm-item');
  const animationDuration = 8000; // 8 seconds
  const now = Date.now();
  const animationProgress = (now % animationDuration) / animationDuration;

  // Keyframes matching the CSS animation `rotate-llm`
  const keyframes = [
    { start: 0.75, index: 3 }, // 75-95%
    { start: 0.50, index: 2 }, // 50-70%
    { start: 0.25, index: 1 }, // 25-45%
    { start: 0.00, index: 0 }  // 0-20%
  ];
  const currentKeyframe = keyframes.find(kf => animationProgress >= kf.start);
  const currentIndex = currentKeyframe ? currentKeyframe.index : 0;

  const currentLLM = llmItems[currentIndex]?.textContent || 'Claude';

  // Build the full prompt
  const prompt = `${currentLLM}, use github.com/kmay89/SCQCS as a template to build me a secure website about `;

  navigator.clipboard.writeText(prompt).then(() => {
    button.classList.add('copied');

    // Reset after 2 seconds
    setTimeout(() => {
      button.classList.remove('copied');
    }, 2000);
  }).catch(err => {
    // Fallback for older browsers
    const textarea = document.createElement('textarea');
    textarea.value = prompt;
    textarea.style.position = 'fixed';
    textarea.style.opacity = '0';
    document.body.appendChild(textarea);
    textarea.select();
    document.execCommand('copy');
    document.body.removeChild(textarea);

    button.classList.add('copied');
    setTimeout(() => {
      button.classList.remove('copied');
    }, 2000);
  });
}

// Attach event listeners to copy buttons
document.querySelectorAll('.ai-prompt-copy').forEach(button => {
  button.addEventListener('click', () => copyPrompt(button));
});
