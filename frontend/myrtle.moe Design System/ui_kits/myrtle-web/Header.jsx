// Header — recreation of frontend/src/components/header/Header.tsx @ v3.
const { Button, Separator, Kbd } = window;

const TOOLS_MENU = [
  { id: "dps",   label: "DPS charts",             desc: "Interactive damage curves per skill",  icon: "chart", kb: ["⌘","D"] },
  { id: "recr",  label: "Recruitment calculator", desc: "Guaranteed tag combos · 1h parity",    icon: "calc",  kb: ["⌘","R"] },
  { id: "tier",  label: "Tier lists",             desc: "Community ranks · live voting",        icon: "star",  kb: ["⌘","T"] },
  { id: "rand",  label: "Randomizer",             desc: "Pick a squad, break the meta",          icon: "dice",  kb: ["⌘","Z"] },
  { id: "pack",  label: "Pack opener",            desc: "Simulate headhunt odds",                icon: "pack",  kb: null     },
];

function ToolsIcon({ name }) {
  const p = { fill: "none", stroke: "currentColor", strokeWidth: 1.9, strokeLinecap: "round", strokeLinejoin: "round" };
  switch (name) {
    case "chart": return <svg viewBox="0 0 24 24" {...p}><path d="M3 3v18h18"/><path d="m19 9-5 5-4-4-3 3"/></svg>;
    case "calc":  return <svg viewBox="0 0 24 24" {...p}><rect x="4" y="3" width="16" height="18" rx="2"/><path d="M8 7h8M8 12h2M14 12h2M8 17h2M14 17h2"/></svg>;
    case "star":  return <svg viewBox="0 0 24 24" {...p}><path d="m12 2 3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/></svg>;
    case "dice":  return <svg viewBox="0 0 24 24" {...p}><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M8 8h.01M16 8h.01M8 16h.01M16 16h.01M12 12h.01"/></svg>;
    case "pack":  return <svg viewBox="0 0 24 24" {...p}><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><path d="M3.3 7 12 12l8.7-5"/><path d="M12 22V12"/></svg>;
    default: return null;
  }
}

function Header({ active, onNav, onOpenCommand }) {
  const items = [
    { href: "/",          label: "Home" },
    { href: "/operators", label: "Operators" },
    { href: "/tools",     label: "Tools", hasMenu: true },
    { href: "/about",     label: "About" },
  ];
  const [openMenu, setOpenMenu] = React.useState(null); // href of open menu
  const closeTimer = React.useRef(null);

  const openNow = (href) => {
    if (closeTimer.current) clearTimeout(closeTimer.current);
    setOpenMenu(href);
  };
  const closeSoon = () => {
    if (closeTimer.current) clearTimeout(closeTimer.current);
    closeTimer.current = setTimeout(() => setOpenMenu(null), 140);
  };

  return (
    <header className="site-header">
      <div className="header-inner">
        <a className="brand" onClick={() => onNav("/")}>
          <span className="brand-bezel" aria-hidden="true">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M4 18c2.5 0 2.5-4 5-4s2.5 4 5 4 2.5-4 5-4"/>
              <path d="M4 12c2.5 0 2.5-4 5-4s2.5 4 5 4 2.5-4 5-4"/>
            </svg>
          </span>
          <span className="brand-word">myrtle.moe</span>
          <span className="brand-tag">v3</span>
        </a>

        <nav className="main-nav">
          {items.map(i => {
            const isOpen = openMenu === i.href;
            return (
              <div
                key={i.href}
                className={`nav-item ${i.hasMenu ? "has-menu" : ""}`}
                onMouseEnter={i.hasMenu ? () => openNow(i.href) : undefined}
                onMouseLeave={i.hasMenu ? closeSoon : undefined}
              >
                <a
                  className={`nav-link ${active === i.href ? "is-active" : ""} ${isOpen ? "is-open" : ""}`}
                  onClick={(e) => {
                    e.preventDefault();
                    if (i.hasMenu) { setOpenMenu(isOpen ? null : i.href); }
                    else { onNav(i.href); }
                  }}
                  href={i.href}
                  aria-haspopup={i.hasMenu ? "menu" : undefined}
                  aria-expanded={i.hasMenu ? isOpen : undefined}
                >
                  {i.label}
                  {i.hasMenu && (
                    <svg className={`chev ${isOpen ? "is-open" : ""}`} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="m6 9 6 6 6-6"/></svg>
                  )}
                </a>
                {i.hasMenu && isOpen && (
                  <div className="nav-menu" role="menu" onMouseEnter={() => openNow(i.href)} onMouseLeave={closeSoon}>
                    <div className="nav-menu-label">Tools</div>
                    <div className="nav-menu-list">
                      {TOOLS_MENU.map(t => (
                        <a key={t.id} className="nav-menu-item" role="menuitem">
                          <span className="nav-menu-icon"><ToolsIcon name={t.icon} /></span>
                          <span className="nav-menu-text">
                            <span className="nav-menu-title">{t.label}</span>
                            <span className="nav-menu-desc">{t.desc}</span>
                          </span>
                          {t.kb && (
                            <span className="nav-menu-kbd">
                              {t.kb.map((k, j) => <Kbd key={j}>{k}</Kbd>)}
                            </span>
                          )}
                        </a>
                      ))}
                    </div>
                    <div className="nav-menu-foot">
                      <span className="small-mono">Press <Kbd>⌘</Kbd><Kbd>K</Kbd> for all commands</span>
                      <a className="pill-link" onClick={() => { setOpenMenu(null); onOpenCommand(); }}>Open palette →</a>
                    </div>
                  </div>
                )}
              </div>
            );
          })}
        </nav>

        <div className="header-actions">
          <button className="coss-btn v-outline s-sm header-search" onClick={onOpenCommand}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/></svg>
            <span>Search operators…</span>
            <span className="kbd-inline"><kbd className="coss-kbd">⌘</kbd><kbd className="coss-kbd">K</kbd></span>
          </button>
          <Separator orientation="vertical" className="h5" />
          <Button variant="ghost" size="icon" className="theme-btn">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
            </svg>
          </Button>
          <Button variant="ghost" size="icon" className="gh-btn">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4"/><path d="M9 18c-4.51 2-5-2-7-2"/></svg>
          </Button>
          <Button variant="default" size="sm">Sign in</Button>
        </div>
      </div>
    </header>
  );
}
Object.assign(window, { Header });
