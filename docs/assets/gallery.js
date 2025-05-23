/*  Simple vanilla-JS lightbox for the gallery  */
(function () {
  function initItem(item) {
    const overlay = item.querySelector('.overlay');
    const close   = overlay.querySelector('.close');

    const open  = () => { item.classList.add('active'); };
    const closeO = () => { item.classList.remove('active'); };

    item.addEventListener('click', open);
    item.addEventListener('keydown', e => {
      if ((e.key === 'Enter' || e.key === ' ') && !item.classList.contains('active')) {
        e.preventDefault(); open();
      }
    });
    close.addEventListener('click', e => { e.stopPropagation(); closeO(); });
    overlay.addEventListener('click', closeO);
    document.addEventListener('keydown', e => {
      if (e.key === 'Escape' && item.classList.contains('active')) closeO();
    });
  }

  document.addEventListener('DOMContentLoaded', () => {
    document.querySelectorAll('.gallery-item').forEach(initItem);
  });
})();
