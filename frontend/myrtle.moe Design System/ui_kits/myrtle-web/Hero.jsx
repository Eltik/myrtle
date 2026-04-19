// Hero — v3 redesign centered on COSS UI Command palette as the focal element.
const { Button, Badge, Kbd, Kicker } = window;

// Story-CG options for the hero backdrop. Each is 16:9, darkened/overlaid in CSS.
const HERO_CGS = {
  skadi:   { src: "art/cg-skadi.png",   label: "STULTIFERA · SKADI" },
  wcrits:  { src: "art/cg-wcrits.png",  label: "HORTUS · W × CRITS" },
  gavial:  { src: "art/cg-gavial.png",  label: "GUIDING AHEAD · GAVIAL" },
};

// Interactive 3D tilt on the command-palette preview.
// Follows mouse with parallax and lifts the card on the Z axis.
// Rest pose matches the original decorative tilt (rotateX 6deg / rotateY -4deg).
function useHeroTilt() {
  const ref = React.useRef(null);
  React.useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const MAX_X = 12, MAX_Y = 14, LIFT = 24;
    let raf = 0;
    let tx = 6, ty = -4, tz = 0;
    let dx = 6, dy = -4, dz = 0;
    const step = () => {
      tx += (dx - tx) * 0.18;
      ty += (dy - ty) * 0.18;
      tz += (dz - tz) * 0.18;
      el.style.setProperty("--tilt-x", tx.toFixed(2) + "deg");
      el.style.setProperty("--tilt-y", ty.toFixed(2) + "deg");
      el.style.setProperty("--tilt-z", tz.toFixed(1) + "px");
      if (Math.abs(dx - tx) > 0.05 || Math.abs(dy - ty) > 0.05 || Math.abs(dz - tz) > 0.2) {
        raf = requestAnimationFrame(step);
      } else { raf = 0; }
    };
    const onMove = (e) => {
      const r = el.getBoundingClientRect();
      const nx = (e.clientX - r.left) / r.width  - 0.5;
      const ny = (e.clientY - r.top)  / r.height - 0.5;
      dy = -nx * MAX_Y;
      dx =  ny * MAX_X;
      dz = LIFT;
      el.classList.add("is-tilting");
      if (!raf) raf = requestAnimationFrame(step);
    };
    const onLeave = () => {
      dx = 6; dy = -4; dz = 0;
      el.classList.remove("is-tilting");
      if (!raf) raf = requestAnimationFrame(step);
    };
    el.addEventListener("mousemove", onMove);
    el.addEventListener("mouseleave", onLeave);
    return () => {
      el.removeEventListener("mousemove", onMove);
      el.removeEventListener("mouseleave", onLeave);
      if (raf) cancelAnimationFrame(raf);
    };
  }, []);
  return ref;
}

function Hero({ onOpenCommand, onExplore, heroCg = "skadi" }) {
  const tiltRef = useHeroTilt();
  const cg = HERO_CGS[heroCg] || HERO_CGS.skadi;
  return (
    <section className="hero">
      <div className="hero-bg" aria-hidden="true">
        <img className="hero-bg-img" src={cg.src} alt="" loading="eager" decoding="async" key={heroCg} />
      </div>
      <div className="hero-glow" aria-hidden="true" />
      <div className="hero-grid" aria-hidden="true" />
      <div className="hero-cg-credit" aria-hidden="true">
        <span>{cg.label}</span>
      </div>

      <div className="hero-inner">
        <div className="hero-copy">
          <div className="status-pill">
            <span className="dot-pulse" aria-hidden="true" />
            <span className="small-mono">v3 · live backend sync</span>
            <Separator orientation="vertical" className="h3" />
            <a className="pill-link">changelog →</a>
          </div>

          <h1 className="display">
            The <span className="accent">Arknights</span> companion,<br/>
            rebuilt for speed.
          </h1>

          <p className="lead">
            400+ operators, complete stats, community tier lists, and live roster sync.
            Hit <Kbd>⌘</Kbd> <Kbd>K</Kbd> to jump anywhere.
          </p>

          <div className="hero-actions">
            <Button variant="default" size="lg" onClick={onOpenCommand}>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/></svg>
              Search operators
            </Button>
            <Button variant="outline" size="lg" onClick={onExplore}>
              Link Yostar account
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M5 12h14"/><path d="m12 5 7 7-7 7"/></svg>
            </Button>
          </div>

          <div className="hero-stats">
            <div><b>423</b><span>operators</span></div>
            <div><b>58</b><span>tier lists</span></div>
            <div><b>1.2M</b><span>rosters synced</span></div>
          </div>
        </div>

        <div className="hero-panel" ref={tiltRef} onClick={onOpenCommand} role="button" tabIndex={0}>
          <CommandPreview />
        </div>
      </div>
    </section>
  );
}

// A static visual preview of the COSS UI Command component — modeled after v3's command.tsx
function CommandPreview() {
  return (
    <div className="cmd-popup">
      <div className="cmd-head">
        <svg className="cmd-search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/></svg>
        <input className="cmd-input" defaultValue="myr" placeholder="Type a command or search…" />
        <Kbd>esc</Kbd>
      </div>
      <div className="cmd-panel">
        <div className="cmd-group">
          <div className="cmd-group-label">Operators</div>
          <div className="cmd-item is-highlighted">
            <span className="op-chip c-vanguard">V</span>
            <span className="cmd-primary">Myrtle</span>
            <span className="cmd-meta">4★ · Vanguard · Tactician</span>
            <Kbd>↵</Kbd>
          </div>
          <div className="cmd-item">
            <span className="op-chip c-caster">C</span>
            <span className="cmd-primary">Mostima</span>
            <span className="cmd-meta">6★ · Caster · Mystic</span>
          </div>
          <div className="cmd-item">
            <span className="op-chip c-medic">M</span>
            <span className="cmd-primary">Myrrh</span>
            <span className="cmd-meta">4★ · Medic · Medic</span>
          </div>
        </div>
        <div className="cmd-sep" />
        <div className="cmd-group">
          <div className="cmd-group-label">Tools</div>
          <div className="cmd-item">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18M9 21V9"/></svg>
            <span className="cmd-primary">Recruitment calculator</span>
            <Kbd>⌘</Kbd><Kbd>R</Kbd>
          </div>
          <div className="cmd-item">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M3 3v18h18"/><path d="m19 9-5 5-4-4-3 3"/></svg>
            <span className="cmd-primary">DPS charts</span>
            <Kbd>⌘</Kbd><Kbd>D</Kbd>
          </div>
        </div>
      </div>
      <div className="cmd-footer">
        <span><Kbd>↑</Kbd><Kbd>↓</Kbd> to navigate</span>
        <span><Kbd>↵</Kbd> to select</span>
        <span className="right">powered by COSS UI</span>
      </div>
    </div>
  );
}

Object.assign(window, { Hero, CommandPreview });
