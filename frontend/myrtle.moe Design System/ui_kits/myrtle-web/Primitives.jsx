// COSS-UI-flavored primitives (cosmetic recreation)
// Mirrors Eltik/myrtle@v3 ui/button.tsx, ui/badge.tsx, ui/card.tsx, ui/kbd.tsx, ui/separator.tsx.

function Button({ children, variant = "default", size = "default", onClick, className = "", render }) {
  const base = "coss-btn";
  const v = `v-${variant}`;
  const s = `s-${size}`;
  if (render) {
    return React.cloneElement(render, {
      className: [base, v, s, className, render.props.className].filter(Boolean).join(" "),
      onClick,
    });
  }
  return (
    <button type="button" onClick={onClick} className={[base, v, s, className].filter(Boolean).join(" ")}>
      {children}
    </button>
  );
}

function Badge({ children, variant = "default" }) {
  return <span className={`coss-badge v-${variant}`}>{children}</span>;
}

function Card({ children, className = "" }) {
  return <div className={`coss-card ${className}`}>{children}</div>;
}

function Kbd({ children }) {
  return <kbd className="coss-kbd">{children}</kbd>;
}

function Separator({ orientation = "horizontal", className = "" }) {
  return <div role="separator" className={`coss-sep ${orientation} ${className}`} />;
}

function Kicker({ children }) {
  return <span className="coss-kicker">{children}</span>;
}

Object.assign(window, { Button, Badge, Card, Kbd, Separator, Kicker });
