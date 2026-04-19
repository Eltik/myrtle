// Full CommandDialog overlay — toggled on ⌘K or hero click.
const { Kbd } = window;

function CommandDialog({ open, onClose }) {
  const [q, setQ] = React.useState("");
  const inputRef = React.useRef(null);

  React.useEffect(() => {
    if (!open) return;
    setQ("");
    const t = setTimeout(() => inputRef.current?.focus(), 60);
    const onKey = (e) => { if (e.key === "Escape") onClose(); };
    window.addEventListener("keydown", onKey);
    return () => { clearTimeout(t); window.removeEventListener("keydown", onKey); };
  }, [open, onClose]);

  if (!open) return null;

  const operators = [
    { id: "myrtle", name: "Myrtle", role: "Vanguard", meta: "4★ · Tactician", cls: "c-vanguard", letter: "M" },
    { id: "mostima", name: "Mostima", role: "Caster", meta: "6★ · Mystic", cls: "c-caster", letter: "M" },
    { id: "myrrh", name: "Myrrh", role: "Medic", meta: "4★ · Medic", cls: "c-medic", letter: "M" },
    { id: "saria", name: "Saria", role: "Defender", meta: "6★ · Guardian", cls: "c-defender", letter: "S" },
    { id: "suzuran", name: "Suzuran", role: "Supporter", meta: "6★ · Hexer", cls: "c-support", letter: "S" },
  ];
  const tools = [
    { id: "r", label: "Recruitment calculator", shortcut: ["⌘","R"], icon: "calc" },
    { id: "d", label: "DPS charts",             shortcut: ["⌘","D"], icon: "chart" },
    { id: "t", label: "Tier lists",             shortcut: ["⌘","T"], icon: "star" },
    { id: "z", label: "Randomizer",             shortcut: ["⌘","Z"], icon: "dice" },
  ];
  const filtered = q
    ? operators.filter(o => o.name.toLowerCase().includes(q.toLowerCase()))
    : operators;
  const filteredTools = q
    ? tools.filter(t => t.label.toLowerCase().includes(q.toLowerCase()))
    : tools;

  return (
    <div className="cmd-backdrop" onClick={onClose}>
      <div className="cmd-viewport">
        <div className="cmd-popup live" onClick={e => e.stopPropagation()}>
          <div className="cmd-head">
            <svg className="cmd-search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/></svg>
            <input
              ref={inputRef}
              className="cmd-input"
              value={q}
              onChange={(e) => setQ(e.target.value)}
              placeholder="Type a command or search…"
            />
            <Kbd>esc</Kbd>
          </div>
          <div className="cmd-panel">
            {filtered.length > 0 && (
              <div className="cmd-group">
                <div className="cmd-group-label">Operators</div>
                {filtered.map((o, i) => (
                  <div className={`cmd-item ${i === 0 ? "is-highlighted" : ""}`} key={o.id}>
                    <span className={`op-chip ${o.cls}`}>{o.letter}</span>
                    <span className="cmd-primary">{o.name}</span>
                    <span className="cmd-meta">{o.meta}</span>
                    {i === 0 && <Kbd>↵</Kbd>}
                  </div>
                ))}
              </div>
            )}
            {filteredTools.length > 0 && (
              <React.Fragment>
                <div className="cmd-sep" />
                <div className="cmd-group">
                  <div className="cmd-group-label">Tools</div>
                  {filteredTools.map(t => (
                    <div className="cmd-item" key={t.id}>
                      <ToolIcon name={t.icon} />
                      <span className="cmd-primary">{t.label}</span>
                      {t.shortcut.map((k, j) => <Kbd key={j}>{k}</Kbd>)}
                    </div>
                  ))}
                </div>
              </React.Fragment>
            )}
            {filtered.length === 0 && filteredTools.length === 0 && (
              <div className="cmd-empty">No results for "{q}".</div>
            )}
          </div>
          <div className="cmd-footer">
            <span><Kbd>↑</Kbd><Kbd>↓</Kbd> to navigate</span>
            <span><Kbd>↵</Kbd> to select</span>
            <span className="right">powered by COSS UI</span>
          </div>
        </div>
      </div>
    </div>
  );
}

function ToolIcon({ name }) {
  const p = { fill: "none", stroke: "currentColor", strokeWidth: 2, strokeLinecap: "round", strokeLinejoin: "round" };
  switch (name) {
    case "calc":  return <svg viewBox="0 0 24 24" {...p}><rect x="4" y="3" width="16" height="18" rx="2"/><path d="M8 7h8M8 12h2M14 12h2M8 17h2M14 17h2"/></svg>;
    case "chart": return <svg viewBox="0 0 24 24" {...p}><path d="M3 3v18h18"/><path d="m19 9-5 5-4-4-3 3"/></svg>;
    case "star":  return <svg viewBox="0 0 24 24" {...p}><path d="m12 2 3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/></svg>;
    case "dice":  return <svg viewBox="0 0 24 24" {...p}><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M8 8h.01M16 8h.01M8 16h.01M16 16h.01M12 12h.01"/></svg>;
    default: return null;
  }
}

function Footer() {
  return (
    <footer className="site-footer">
      <div className="footer-inner page-wrap">
        <div className="footer-brand">
          <span className="brand-bezel small">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round"><path d="M4 18c2.5 0 2.5-4 5-4s2.5 4 5 4 2.5-4 5-4"/><path d="M4 12c2.5 0 2.5-4 5-4s2.5 4 5 4 2.5-4 5-4"/></svg>
          </span>
          <span>myrtle.moe <span className="muted">· v3</span></span>
        </div>
        <span className="footer-note">Not affiliated with Hypergryph or Yostar. Game data and assets are property of their respective owners.</span>
        <span className="footer-meta mono">built on TanStack Start · COSS UI</span>
      </div>
    </footer>
  );
}

Object.assign(window, { CommandDialog, Footer });
