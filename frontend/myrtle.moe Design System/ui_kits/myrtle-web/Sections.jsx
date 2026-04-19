const { Card, Badge, Kicker, Button } = window;

// ─── Features strip ─────────────────────────────────────────────
function FeatureStrip() {
  const items = [
    { icon: "database", k: "Live data", t: "400+ operators, instant search", d: "Skills, talents, modules, skins, voice lines. Filter by faction, archetype, or tag.", accent: "coral" },
    { icon: "sync",     k: "Roster sync", t: "Your account, mirrored",       d: "Link a Yostar account and see your live box, E2 progress, base layout in real time.", accent: "mint" },
    { icon: "bolt",     k: "Tools",     t: "DPS, recruit, randomizer",        d: "Interactive calculators and community-maintained tier lists for every stage meta.",   accent: "amber" },
  ];
  return (
    <section className="features page-wrap">
      {items.map((f, i) => (
        <div className={`feat-card accent-${f.accent}`} key={i}>
          <div className="feat-icon"><FeatIcon name={f.icon} /></div>
          <Kicker>{f.k}</Kicker>
          <div className="feat-title">{f.t}</div>
          <p className="feat-desc">{f.d}</p>
          <a className="feat-link">Explore <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M5 12h14"/><path d="m12 5 7 7-7 7"/></svg></a>
        </div>
      ))}
    </section>
  );
}

function FeatIcon({ name }) {
  const p = { fill: "none", stroke: "currentColor", strokeWidth: 1.8, strokeLinecap: "round", strokeLinejoin: "round" };
  switch (name) {
    case "database": return <svg viewBox="0 0 24 24" {...p}><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v6c0 1.66 4 3 9 3s9-1.34 9-3V5"/><path d="M3 11v6c0 1.66 4 3 9 3s9-1.34 9-3v-6"/></svg>;
    case "sync":     return <svg viewBox="0 0 24 24" {...p}><path d="M21 12a9 9 0 0 0-15-6.7L3 8"/><path d="M3 4v4h4"/><path d="M3 12a9 9 0 0 0 15 6.7l3-2.7"/><path d="M21 20v-4h-4"/></svg>;
    case "bolt":     return <svg viewBox="0 0 24 24" {...p}><path d="M13 2 3 14h9l-1 8 10-12h-9l1-8z"/></svg>;
    default: return null;
  }
}

const ROLE_CLS = { Vanguard: "c-vanguard", Supporter: "c-support", Defender: "c-defender", Sniper: "c-sniper", Caster: "c-caster", Medic: "c-medic", Guard: "c-guard", Specialist: "c-special" };

// Operator records (compact) used as targets of the drawer + chips in tier lists.
const OPERATORS = [
  { id: "texas",   name: "Texas Alter", rarity: 6, role: "Guard",      arch: "Reaper" },
  { id: "mudrock", name: "Mudrock",     rarity: 6, role: "Defender",   arch: "Arts Protector" },
  { id: "suzuran", name: "Suzuran",     rarity: 6, role: "Supporter",  arch: "Hexer" },
  { id: "sura",    name: "Surtr",       rarity: 6, role: "Guard",      arch: "Dreadnought" },
  { id: "exu",     name: "Exusiai",     rarity: 6, role: "Sniper",     arch: "Marksman" },
  { id: "eyja",    name: "Eyjafjalla",  rarity: 6, role: "Caster",     arch: "Core Caster" },
  { id: "sara",    name: "Saria",       rarity: 6, role: "Defender",   arch: "Guardian" },
  { id: "kal",     name: "Kal'tsit",    rarity: 6, role: "Medic",      arch: "Summoner" },
  { id: "ceo",     name: "Ceobe",       rarity: 6, role: "Caster",     arch: "Core Caster" },
  { id: "thorns",  name: "Thorns",      rarity: 6, role: "Guard",      arch: "Lord" },
  { id: "myrtle",  name: "Myrtle",      rarity: 4, role: "Vanguard",   arch: "Tactician" },
  { id: "gg",      name: "Goldenglow",  rarity: 6, role: "Caster",     arch: "Phalanx" },
  { id: "ptila",   name: "Ptilopsis",   rarity: 5, role: "Medic",      arch: "Wandering" },
  { id: "blaze",   name: "Blaze",       rarity: 6, role: "Guard",      arch: "Centurion" },
  { id: "ho",      name: "Hoshiguma",   rarity: 6, role: "Defender",   arch: "Guardian" },
  { id: "silver",  name: "SilverAsh",   rarity: 6, role: "Guard",      arch: "Lord" },
];
const OP_BY_ID = Object.fromEntries(OPERATORS.map(o => [o.id, o]));

// ─── Tier lists section ─────────────────────────────────────────
// Replaces the former "Your roster, live" grid. Community tier lists
// rendered as cards with a mini S/A/B strip using role-colored chips.
const TIER_LISTS = [
  {
    id: "endgame-2026",
    title: "Endgame Stages · Current Meta",
    tag: "Endgame",
    stage: "IS#4 · CC#14 · Annihilation",
    author: { name: "AceShippo", initials: "AS" },
    updated: "2h ago",
    votes: 1284,
    comments: 312,
    hot: true,
    accent: "coral",
    tiers: {
      S: ["texas", "mudrock", "suzuran", "sura"],
      A: ["exu", "eyja", "sara", "kal"],
      B: ["ceo", "thorns", "myrtle"],
    },
  },
  {
    id: "beginner-6mo",
    title: "Starter Box · First 6 Months",
    tag: "Beginner",
    stage: "Main Theme · Chapter 1–8",
    author: { name: "Perro_Salchicha", initials: "PS" },
    updated: "yesterday",
    votes: 842,
    comments: 198,
    accent: "mint",
    tiers: {
      S: ["myrtle", "ptila", "suzuran"],
      A: ["exu", "blaze", "ho"],
      B: ["silver", "sara"],
    },
  },
  {
    id: "is4-relic",
    title: "Integrated Strategies #4",
    tag: "Roguelike",
    stage: "Expeditioner's Joklumarkar",
    author: { name: "Viktor R.",  initials: "VR" },
    updated: "3d ago",
    votes: 516,
    comments: 71,
    accent: "amber",
    tiers: {
      S: ["gg", "thorns", "mudrock"],
      A: ["texas", "sura", "kal"],
      B: ["ceo", "eyja"],
    },
  },
  {
    id: "arts-dps",
    title: "Arts DPS Showdown",
    tag: "Niche",
    stage: "High-RES bosses, sarkaz crawls",
    author: { name: "dps_nerd",   initials: "DN" },
    updated: "last week",
    votes: 394,
    comments: 88,
    accent: "violet",
    tiers: {
      S: ["eyja", "gg", "ceo"],
      A: ["kal", "suzuran"],
      B: ["ptila"],
    },
  },
];

function TierLists({ onSelect }) {
  const [filter, setFilter] = React.useState("All");
  const filters = ["All", "Endgame", "Beginner", "Roguelike", "Niche"];
  const list = TIER_LISTS.filter(t => filter === "All" ? true : t.tag === filter);

  return (
    <section className="operators page-wrap" data-screen-label="03 Tier lists">
      <div className="section-head">
        <div className="section-head-row">
          <div>
            <Kicker>Community</Kicker>
            <h2 className="section-title">Tier lists, at a glance.</h2>
            <p className="section-lead">Previews from the 24 most-watched lists. Click any card to open the full ranking.</p>
          </div>
          <div className="op-filters" role="tablist">
            {filters.map(f => (
              <button key={f} role="tab" className={`op-chip ${filter === f ? "is-active" : ""}`} onClick={() => setFilter(f)}>
                {f}
              </button>
            ))}
          </div>
        </div>
      </div>

      <div className="tl-grid">
        {list.map(tl => (
          <TierListCard key={tl.id} tl={tl} onSelect={onSelect} />
        ))}
      </div>

      <a className="tl-browse">
        <span className="tl-browse-dot" aria-hidden="true"></span>
        <span className="tl-browse-text">Browse all 24 tier lists</span>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M5 12h14"/><path d="m12 5 7 7-7 7"/></svg>
      </a>
    </section>
  );
}

function TierListCard({ tl, onSelect }) {
  const totalOps = Object.values(tl.tiers).reduce((n, arr) => n + arr.length, 0);
  const topOps = (tl.tiers.S || []).slice(0, 4).map(id => OP_BY_ID[id]).filter(Boolean);
  const restCount = totalOps - topOps.length;
  // Ghost avatars for A/B tiers (role-tinted circles, no labels) to hint at depth
  const ghostOps = [...(tl.tiers.A || []), ...(tl.tiers.B || [])].slice(0, 6).map(id => OP_BY_ID[id]).filter(Boolean);

  return (
    <article className={`tl-card accent-${tl.accent}`} role="link" tabIndex={0}>
      <div className="tl-head">
        <div className="tl-tag">
          <span className="tl-tag-dot" aria-hidden="true"></span>
          <span>{tl.tag}</span>
          {tl.hot && <span className="tl-hot-inline">· trending</span>}
        </div>
        <h3 className="tl-title">{tl.title}</h3>
      </div>

      <div className="tl-preview">
        <div className="tl-preview-head">
          <span className="tl-s-pill">S-tier</span>
          <span className="tl-s-count">{tl.tiers.S.length} picks</span>
        </div>
        <div className="tl-s-row">
          {topOps.map(op => (
            <button
              key={op.id}
              className={`tl-chip ${ROLE_CLS[op.role] || ""}`}
              onClick={(e) => { e.stopPropagation(); onSelect({ ...op, e: 2, lvl: 90, hp: 2020, atk: 510, def: 225, trust: 200, owned: true }); }}
              title={`${op.name} · ${op.role}`}
            >
              <span className="tl-chip-initials">{op.name.split(" ")[0].slice(0,2)}</span>
              <span className="tl-chip-name">{op.name}</span>
            </button>
          ))}
        </div>

        <div className="tl-ghost">
          <span className="tl-ghost-label">+ A, B tiers</span>
          <div className="tl-ghost-stack">
            {ghostOps.map((op, i) => (
              <span key={op.id + i} className={`tl-ghost-dot ${ROLE_CLS[op.role] || ""}`} style={{ ["--i"]: i }} title={op.name}></span>
            ))}
            {restCount > 4 && <span className="tl-ghost-more">+{restCount - 4}</span>}
          </div>
        </div>
      </div>

      <div className="tl-foot">
        <div className="tl-foot-meta">
          <span className="tl-avatar">{tl.author.initials}</span>
          <span className="tl-foot-text">
            <span className="tl-author-name">{tl.author.name}</span>
            <span className="tl-foot-sep">·</span>
            <span className="tl-foot-time">{tl.updated}</span>
          </span>
        </div>
        <div className="tl-foot-cta">
          <span className="tl-votes" title={`${tl.votes} upvotes`}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="m5 15 7-7 7 7"/></svg>
            {tl.votes > 999 ? (tl.votes/1000).toFixed(1) + "k" : tl.votes}
          </span>
          <span className="tl-cta">
            Open
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M5 12h14"/><path d="m12 5 7 7-7 7"/></svg>
          </span>
        </div>
      </div>
    </article>
  );
}

// Drawer still works for operator chips clicked from tier lists
function OperatorDrawer({ op, onClose }) {
  if (!op) return null;
  return (
    <div className="drawer-backdrop" onClick={onClose}>
      <aside className="drawer" onClick={e => e.stopPropagation()}>
        <div className="drawer-head">
          <Kicker>Operator</Kicker>
          <button className="coss-btn v-ghost s-icon" onClick={onClose} aria-label="Close">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M18 6 6 18M6 6l12 12"/></svg>
          </button>
        </div>
        <div className={`drawer-hero ${ROLE_CLS[op.role] || ""}`}>
          <span className="drawer-initials">{op.name.slice(0,2)}</span>
          <span className="drawer-rarity">{"★".repeat(op.rarity)}</span>
        </div>
        <h2 className="drawer-title">{op.name}</h2>
        <div className="drawer-tags">
          <Badge>{op.role}</Badge>
          <Badge variant="secondary">{op.arch}</Badge>
          {op.owned ? <Badge variant="success">E{op.e} · Lv{op.lvl}</Badge> : <Badge variant="outline">Not owned</Badge>}
        </div>
        <div className="stat-grid">
          <div><div className="stat-label">HP</div><div className="stat-val">2,020</div></div>
          <div><div className="stat-label">ATK</div><div className="stat-val">510</div></div>
          <div><div className="stat-label">DEF</div><div className="stat-val">225</div></div>
          <div><div className="stat-label">RES</div><div className="stat-val">0</div></div>
        </div>
        <div className="drawer-actions">
          <Button variant="default">View skills</Button>
          <Button variant="outline">Add to team</Button>
        </div>
      </aside>
    </div>
  );
}

// Keep OperatorGrid name for index.html destructure compat — it now renders TierLists.
const OperatorGrid = TierLists;

Object.assign(window, { FeatureStrip, OperatorGrid, TierLists, OperatorDrawer });
