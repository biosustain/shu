# Welcome to shu's documentation!

**shu** is an interface to plot data on top of metabolic networks,
with emphasis in multi-dimensional data across diferent conditions and distributions.

Shu is available as a web application at [https://biosustain.github.io/shu](https://biosustain.github.io/shu) or as a native
application (see [releases](https://github.com/biosustain/shu/releases/latest)).

Check out the [plotting](plotting.md) section for an overview on how to generate and plot data
into the map. The [File formats](file_formats.md) section explains the map (compatible
with [escher](https://github.com/zakandrewking/escher)) and data especification.

<!-- Controls with live-input highlights — copy into your MkDocs page -->
<style>
:root{
  --bg:#f9fafb; --border:#d1d5db; --radius:12px; --shadow:0 2px 4px rgba(0,0,0,.08);
  --kbd-bg:#fff; --kbd-bd:#cbd5e1; --hi:#ffaa00;        /* amber highlight */
}

.controls{max-width:640px;margin:2rem auto;font-family:system-ui,-apple-system,Segoe UI,Roboto,sans-serif}
.controls h2{text-align:center;margin-bottom:1.2rem;font-size:1.5rem;font-weight:600}

.control-item{display:flex;align-items:center;gap:1rem;background:var(--bg);border:1px solid var(--border);
              border-radius:var(--radius);box-shadow:var(--shadow);padding:.75rem 1rem;margin:.6rem 0}

/* icons ---------------------------------------------------------*/
.icon{width:46px;height:46px;position:relative;flex:0 0 46px}
.mouse{width:100%;height:100%;border:2px solid #111;border-radius:10px;position:relative}
.mouse::before{content:'';position:absolute;top:4px;bottom:4px;background:#e5e7eb}
.mouse.left::before{left:4px;right:60%}
.mouse.middle::before{left:calc(50% - 5px);width:10px}
.mouse.right::before{right:4px;left:60%}
.wheel::before{left:calc(50% - 3px);width:6px;border-radius:3px}

/* kbd bubble ----------------------------------------------------*/
.kbd{display:inline-block;background:var(--kbd-bg);border:1px solid var(--kbd-bd);border-radius:4px;
     padding:.15rem .4rem;font-size:.8rem;line-height:1.2;font-family:inherit;
     box-shadow:inset 0 -2px var(--kbd-bd);white-space:nowrap}
.icon .kbd{position:absolute;bottom:-12px;right:-10px}

.description{flex:1;line-height:1.4}

/* ===== highlight state ===== */
.active,
.active .mouse,
.active .kbd{outline:3px solid var(--hi);outline-offset:2px}
</style>

<section class="controls">
  <h2>Map Controls</h2>

  <div class="control-item" id="mouse-left">
    <div class="icon"><div class="mouse left"></div></div>
    <div class="description">Left-click&nbsp;&amp; drag to move around the map</div>
  </div>

  <div class="control-item" id="mouse-wheel">
    <div class="icon"><div class="mouse wheel"></div></div>
    <div class="description">Scroll wheel to zoom in / out</div>
  </div>

  <div class="control-item" id="mouse-middle">
    <div class="icon"><div class="mouse middle"></div></div>
    <div class="description">Middle-click a histogram or the legend, then drag to move it</div>
  </div>

  <div class="control-item" id="mouse-right">
    <div class="icon"><div class="mouse right"></div></div>
    <div class="description">Right-click a histogram or the legend, then drag to rotate / zoom</div>
  </div>

  <div class="control-item" id="mouse-right-shift">
    <div class="icon">
      <div class="mouse right"></div>
      <span class="kbd kbd-shift">Shift</span>
    </div>
    <div class="description">Right-click + <kbd class="kbd kbd-shift">Shift</kbd> on a histogram, then drag to scale its&nbsp;x-axis</div>
  </div>

  <div class="control-item" id="key-plus-minus">
    <div class="icon">
      <span class="kbd kbd-plus">+</span>
      <span class="kbd kbd-minus" style="margin-left:.15rem;">−</span>
    </div>
    <div class="description">
      <kbd class="kbd kbd-plus">+</kbd> / <kbd class="kbd kbd-minus">−</kbd> to scale the legend.<br>
      Hold <kbd class="kbd kbd-ctrl">Ctrl</kbd> to scale the Settings window
    </div>
  </div>

  <div class="control-item" id="settings-export">
    <div class="icon">
      <span style="font-size:1.7rem;position:absolute;top:50%;left:50%;transform:translate(-50%,-50%);">⚙</span>
    </div>
    <div class="description">
      Use the <strong>Settings</strong> window to change appearance or export the map as JSON / PNG
    </div>
  </div>
</section>

<script>
/* ------------------------------------------------------------- *
 *  INPUT → PANEL HIGHLIGHT                                      *
 * ------------------------------------------------------------- */
(function () {
  /* SHARED UTILS ---------------------------------------------- */
  const on = (el, type, fn) => el.addEventListener(type, fn);
  const add  = (id) => document.getElementById(id)?.classList.add('active');
  const drop = (id) => document.getElementById(id)?.classList.remove('active');

  /* MOUSE / SCROLL -------------------------------------------- */
  const mouseMap = {0:'mouse-left',1:'mouse-middle',2:'mouse-right'};
  on(window,'mousedown',e=>{
    const base = mouseMap[e.button];
    if(!base) return;
    // right-click may be plain or with Shift
    const id = (e.button===2 && e.shiftKey)? 'mouse-right-shift' : base;
    add(id);
  });
  on(window,'mouseup',e=>{
    Object.values(mouseMap).forEach(drop);
    drop('mouse-right-shift');
  });
  on(window,'contextmenu',e=>e.preventDefault());            // keep panel usable

  // Wheel: momentary flash
  on(window,'wheel',e=>{
    add('mouse-wheel');
    clearTimeout(window.__wheelTO);
    window.__wheelTO = setTimeout(()=>drop('mouse-wheel'),200);
  });

  /* KEYBOARD --------------------------------------------------- */
  const keyMap = {
    '+':'key-plus-minus',
    '=':'key-plus-minus',   // some layouts send '=' then Shift produces '+'
    '-':'key-plus-minus',
    'Control':'key-plus-minus',
  };

  on(window,'keydown',e=>{
    const id = keyMap[e.key];
    if(id) add(id);
  });
  on(window,'keyup',e=>{
    const id = keyMap[e.key];
    if(id) drop(id);
  });
})();
</script>

