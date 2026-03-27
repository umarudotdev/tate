document.addEventListener('DOMContentLoaded', function () {
  if (window.innerWidth <= 768) {
    document.querySelectorAll('.sidebar__mobile[open]').forEach(function (d) {
      d.removeAttribute('open');
    });
  }

  var observer = new IntersectionObserver(function (entries) {
    entries.forEach(function (entry) {
      if (entry.isIntersecting) {
        entry.target.classList.add('visible');
        if (entry.target.classList.contains('deck')) {
          entry.target.classList.add('deck--entering');
          setTimeout(function () {
            entry.target.classList.remove('deck--entering');
          }, 500);
        }
        observer.unobserve(entry.target);
      }
    });
  }, { threshold: 0.1 });

  function observeReveals() {
    document.querySelectorAll('.reveal:not(.visible)').forEach(function (el) {
      observer.observe(el);
    });
  }

  observeReveals();

  window.addEventListener('pagereveal', function (e) {
    if (e.viewTransition) {
      document.querySelectorAll('.reveal').forEach(function (el) {
        el.classList.remove('visible');
      });
      e.viewTransition.ready.then(function () {
        observeReveals();
      });
    }
  });

  var copyIcon = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>';
  var checkIcon = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>';

  var deckCards = Array.from(document.querySelectorAll('.deck__card'));
  var hitzone = document.querySelector('.deck__hitzone');
  var activeIdx = 0;
  if (deckCards[0]) deckCards[0].classList.add('deck__card--active');

  function selectCard(idx) {
    if (idx === activeIdx) return;

    deckCards.forEach(function (c) {
      c.classList.remove('deck__card--active', 'flipped');
      var f = c.querySelector('.deck__face--front');
      var b = c.querySelector('.deck__face--back');
      if (f) f.style.transform = '';
      if (b) b.style.transform = '';
    });

    activeIdx = idx;
    deckCards[idx].classList.add('deck__card--active');

    var top = deckCards.length;
    deckCards.forEach(function (c, i) {
      if (i === idx) c.style.zIndex = top;
      else if (i < idx) c.style.zIndex = top - (idx - i);
      else c.style.zIndex = top - i;
    });
  }

  function setTilt(card, rx, ry) {
    var front = card.querySelector('.deck__face--front');
    var back = card.querySelector('.deck__face--back');
    var flipped = card.classList.contains('flipped');
    var tilt = 'rotateX(' + rx + 'deg) rotateY(' + ry + 'deg)';
    if (front) front.style.transform = flipped ? 'rotateY(180deg) ' + tilt : tilt;
    if (back) back.style.transform = flipped ? tilt : 'rotateY(-180deg) ' + tilt;
  }

  function clearTilt() {
    var card = deckCards[activeIdx];
    var front = card.querySelector('.deck__face--front');
    var back = card.querySelector('.deck__face--back');
    if (front) front.style.transform = '';
    if (back) back.style.transform = '';
  }

  if (hitzone) {
    function xToCardIndex(x) {
      var n = deckCards.length;
      var activeWeight = 0.4;
      var otherWeight = (1 - activeWeight) / (n - 1);
      var cursor = 0;
      for (var i = 0; i < n; i++) {
        cursor += (i === activeIdx) ? activeWeight : otherWeight;
        if (x < cursor) return i;
      }
      return n - 1;
    }

    hitzone.addEventListener('mousemove', function (e) {
      var rect = hitzone.getBoundingClientRect();
      var x = (e.clientX - rect.left) / rect.width;
      var idx = xToCardIndex(x);
      selectCard(idx);
      var card = deckCards[activeIdx];
      var cr = card.getBoundingClientRect();
      var cx = (e.clientX - cr.left) / cr.width - 0.5;
      var cy = (e.clientY - cr.top) / cr.height - 0.5;
      setTilt(card, cy * -10, cx * 15);
    });

    hitzone.addEventListener('mouseleave', function () {
      clearTilt();
    });

    hitzone.addEventListener('click', function () {
      var card = deckCards[activeIdx];
      card.classList.toggle('flipped');
      clearTilt();
    });
  }

  var prevBtn = document.querySelector('.deck__arrow--prev');
  var nextBtn = document.querySelector('.deck__arrow--next');
  if (prevBtn) prevBtn.addEventListener('click', function (e) {
    e.stopPropagation();
    selectCard(activeIdx <= 0 ? deckCards.length - 1 : activeIdx - 1);
  });
  if (nextBtn) nextBtn.addEventListener('click', function (e) {
    e.stopPropagation();
    selectCard(activeIdx >= deckCards.length - 1 ? 0 : activeIdx + 1);
  });

  document.querySelectorAll('.terminal__copy').forEach(function (btn) {
    btn.addEventListener('click', function () {
      var text = btn.getAttribute('data-copy');
      navigator.clipboard.writeText(text).then(function () {
        btn.classList.add('terminal__copy--copied');
        btn.replaceChildren();
        btn.insertAdjacentHTML('afterbegin', checkIcon);
        setTimeout(function () {
          btn.classList.remove('terminal__copy--copied');
          btn.replaceChildren();
          btn.insertAdjacentHTML('afterbegin', copyIcon);
        }, 2000);
      });
    });
  });
});
