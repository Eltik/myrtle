import { createFileRoute } from "@tanstack/react-router";
import { seo } from "#/lib/seo";

// A deliberately awful page. Every choice on it is wrong on purpose: Times New
// Roman, WordArt, a marquee, blinking text, a visitor counter, a table with
// border=1. It lives inside the real site chrome because that is the joke.
export const Route = createFileRoute("/help-me")({
    component: HelpMe,
    head: () => {
        const { meta, links } = seo({
            title: "graphic design isnt my passion",
            description: "i need help with ui. please. anyone.",
            path: "/help-me",
            noindex: true,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

const css = `
.hm * { font-family: "Times New Roman", Times, serif !important; box-sizing: content-box; }
.hm {
    color: #000;
    background-color: #ffff99;
    background-image:
        radial-gradient(circle at 20% 30%, rgba(255, 0, 255, 0.18) 0 40px, transparent 41px),
        radial-gradient(circle at 80% 70%, rgba(0, 255, 255, 0.22) 0 60px, transparent 61px),
        repeating-linear-gradient(45deg, transparent 0 28px, rgba(0, 128, 0, 0.08) 28px 30px);
    padding: 12px 16px 60px;
    overflow-x: hidden;
    line-height: 1.15;
}
.hm a { color: #0000ee; }
.hm a:visited { color: #551a8b; }
.hm .wordart {
    font-size: clamp(34px, 8vw, 92px);
    font-weight: bold;
    font-style: italic;
    text-align: center;
    margin: 18px 0 0;
    transform: rotate(-3deg);
    background: linear-gradient(90deg, #f00, #ff8000, #ff0, #0f0, #00f, #8000ff, #f00);
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
    filter: drop-shadow(4px 4px 0 #000) drop-shadow(-2px -2px 0 #fff);
    letter-spacing: -0.02em;
    line-height: 1;
}
.hm .sub { text-align: center; font-size: 26px; margin: 6px 0 0 30px; color: #008000; text-decoration: underline; }
.hm .marquee { overflow: hidden; white-space: nowrap; border: 3px ridge #c0c0c0; background: #fff; margin: 22px -16px 0; }
.hm .marquee span { display: inline-block; padding-left: 100%; font-size: 22px; color: #f00; font-weight: bold; animation: hm-marquee 14s linear infinite; }
@keyframes hm-marquee { from { transform: translateX(0); } to { transform: translateX(-100%); } }
.hm .blink { animation: hm-blink 1s steps(1) infinite; }
@keyframes hm-blink { 50% { opacity: 0; } }
.hm .construction {
    margin: 24px auto 0;
    max-width: 520px;
    padding: 8px 12px;
    text-align: center;
    font-weight: bold;
    font-size: 20px;
    color: #000;
    background: repeating-linear-gradient(135deg, #ffcc00 0 18px, #000 18px 36px);
    text-shadow: 0 0 6px #fff, 0 0 6px #fff, 0 0 6px #fff, 0 0 6px #fff;
    border: 4px double #000;
    transform: rotate(1.5deg);
}
.hm .box {
    border: 3px outset #808080;
    background: #fff;
    padding: 10px 14px;
    margin: 26px 0 0;
    max-width: 640px;
}
.hm .box.left { margin-left: 0; }
.hm .box.right { margin-left: auto; transform: rotate(0.6deg); }
.hm .box.crooked { transform: rotate(-1.2deg); margin-left: 40px; }
.hm h2 { font-size: 30px; margin: 0 0 6px; color: #800080; }
.hm h3 { font-size: 22px; margin: 18px 0 4px; color: #000080; }
.hm p, .hm li, .hm td, .hm th, .hm label { font-size: 18px; }
.hm p { margin: 8px 0; }
.hm table { border-collapse: collapse; width: 100%; }
.hm td, .hm th { border: 1px solid #000; padding: 4px 8px; text-align: left; vertical-align: top; }
.hm th { background: #c0c0c0; }
.hm .big { font-size: 40px; display: inline-block; }
.hm .tilt1 { transform: rotate(12deg); }
.hm .tilt2 { transform: rotate(-20deg); }
.hm .tilt3 { transform: rotate(35deg) translateY(6px); }
.hm .counter { font-family: "Courier New", monospace !important; background: #000; color: #0f0; padding: 2px 6px; letter-spacing: 2px; }
.hm input, .hm textarea, .hm button, .hm select { all: revert; font-size: 16px; }
.hm textarea { width: 100%; max-width: 100%; }
.hm hr { border: 0; border-top: 2px inset #808080; margin: 20px 0; }
.hm .rainbow { background: linear-gradient(90deg, #f00, #ff0, #0f0, #0ff, #00f, #f0f); -webkit-background-clip: text; background-clip: text; color: transparent; font-weight: bold; }
.hm .center { text-align: center; }
.hm .small { font-size: 13px; }
.hm .stamp {
    display: inline-block;
    border: 3px solid #f00;
    color: #f00;
    font-weight: bold;
    font-size: 26px;
    padding: 2px 10px;
    transform: rotate(-14deg);
    text-transform: uppercase;
    letter-spacing: 3px;
    margin: 10px 0 0 60%;
    opacity: 0.85;
}
.hm .box.help { max-width: 720px; margin: 30px auto 0; border: 6px ridge #f00; background: #fff; transform: rotate(-0.8deg); }
.hm .discord {
    display: inline-block;
    font-size: clamp(20px, 4vw, 34px);
    font-weight: bold;
    color: #fff !important;
    background: #5865f2;
    border: 5px outset #7289da;
    padding: 12px 22px;
    text-decoration: none;
    box-shadow: 6px 6px 0 #000;
    animation: hm-wobble 1.6s ease-in-out infinite;
}
.hm .discord:hover { background: #ff00ff; border-color: #ff80ff; }
@keyframes hm-wobble { 0%, 100% { transform: rotate(-2deg) scale(1); } 50% { transform: rotate(2deg) scale(1.04); } }
.hm .guest { border-left: 6px solid #ff00ff; padding-left: 10px; margin: 10px 0; }
.hm .guest b { color: #ff00ff; }
.hm .footer { text-align: center; margin-top: 40px; font-size: 14px; color: #444; }
.hm .footer .ie { display: inline-block; border: 2px groove #808080; padding: 2px 8px; background: #e0e0e0; margin-top: 8px; }
@media (prefers-reduced-motion: reduce) {
    .hm .marquee span { animation: none; padding-left: 0; }
    .hm .blink { animation: none; }
    .hm .discord { animation: none; }
}
`;

function HelpMe() {
    return (
        <div className="hm">
            <style>{css}</style>

            <h1 className="wordart">graphic design isnt my passion</h1>
            <p className="sub">
                i need help with ui <span className="blink">!!!</span>
            </p>

            <div className="marquee" aria-hidden="true">
                <span>~*~*~ welcome to my web page ~*~*~ if u know css please join the discord ~*~*~ link is below ~*~*~ please ~*~*~ i have tried everything ~*~*~ the div is not centering ~*~*~</span>
            </div>

            <div className="construction">🚧 UNDER CONSTRUCTION 🚧 (since 2019)</div>

            <p className="center" style={{ marginTop: 22, fontSize: 20 }}>
                you are visitor number <span className="counter">0000007</span> <span className="small">(hi mom)</span>
            </p>

            <div className="box left">
                <h2>
                    <span className="big tilt1">👋</span> about this page
                </h2>
                <p>
                    hello and welcome. this page was made by me. i am a <b>backend developer</b>. i know what a mutex is. i do not know what "visual hierarchy" is and at this point im too afraid to ask.
                </p>
                <p>
                    everything on this website that looks nice was done by someone else or by accident. this page is what happens when i am left <u>unsupervised</u>.
                </p>
                <p>
                    <span className="rainbow">Times New Roman</span> is the font because it is the font. it was already there. why would i change it.
                </p>
                <div className="stamp">ship it</div>
            </div>

            <div className="box right">
                <h2>
                    <span className="big tilt2">📋</span> things i have tried
                </h2>
                <table>
                    <thead>
                        <tr>
                            <th>#</th>
                            <th>what i did</th>
                            <th>did it work</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>1</td>
                            <td>centered everything</td>
                            <td>no</td>
                        </tr>
                        <tr>
                            <td>2</td>
                            <td>added a gradient</td>
                            <td>made it worse</td>
                        </tr>
                        <tr>
                            <td>3</td>
                            <td>used 14 different fonts</td>
                            <td>coworker cried</td>
                        </tr>
                        <tr>
                            <td>4</td>
                            <td>comic sans</td>
                            <td>banned by HR</td>
                        </tr>
                        <tr>
                            <td>5</td>
                            <td>margin: auto</td>
                            <td>the div moved but not where i wanted</td>
                        </tr>
                        <tr>
                            <td>6</td>
                            <td>!important on everything</td>
                            <td>yes actually</td>
                        </tr>
                        <tr>
                            <td>7</td>
                            <td>asked an AI</td>
                            <td>it said "consider using a design system". i AM the design system</td>
                        </tr>
                        <tr>
                            <td>8</td>
                            <td>drop shadow on the drop shadow</td>
                            <td>👍</td>
                        </tr>
                    </tbody>
                </table>
            </div>

            <div className="box crooked">
                <h2>
                    <span className="big tilt3">❓</span> frequently asked question
                </h2>
                <h3>Q: why is the button not aligned with the input</h3>
                <p>A: i dont know</p>
                <h3>Q: what is the difference between padding and margin</h3>
                <p>A: one of them is on the inside i think</p>
                <h3>Q: can u make the logo bigger</h3>
                <p>A: yes. this is the only request i can fulfill. here:</p>
                <p style={{ fontSize: 64, margin: 0, lineHeight: 1 }}>LOGO</p>
                <h3>Q: what color is the primary color</h3>
                <p>
                    A: <span style={{ color: "#ff00ff" }}>#ff00ff</span>. it was the first one in the picker.
                </p>
            </div>

            <hr />

            <div className="box help">
                <h2 className="center" style={{ fontSize: 36, color: "#f00" }}>
                    <span className="blink">⚠️</span> I NEED HELP <span className="blink">⚠️</span>
                </h2>
                <p className="center" style={{ fontSize: 22 }}>
                    this is not a bit. ok it is a bit. but also i genuinely need someone who knows what they are doing to look at the ui on this site.
                </p>
                <p className="center" style={{ fontSize: 22 }}>
                    if u know design, css, figma, or just have <b>opinions about where buttons should go</b>, please come tell me. i will listen. i will not argue. i will say "oh that makes sense" a lot.
                </p>
                <p className="center" style={{ margin: "22px 0 10px" }}>
                    <a className="discord" href="/discord" target="_blank" rel="noreferrer">
                        👉 CLICK HERE TO JOIN THE DISCORD SERVER 👈
                    </a>
                </p>
                <p className="center small">(it opens in a new tab. that is the only thing i got right on this page)</p>
                <p className="center" style={{ fontSize: 18 }}>
                    once ur in, say <span className="counter">i can help with ui</span> and i will appear. like beetlejuice but for flexbox.
                </p>
                <p className="center small">there is no form here. there was one. it sent to nobody. the discord goes to an actual person (me)</p>
            </div>

            <div className="box right" style={{ maxWidth: 520 }}>
                <h2>📖 guestbook</h2>
                <div className="guest">
                    <b>xXdesignerXx</b> wrote: this hurts to look at. 10/10
                </div>
                <div className="guest">
                    <b>sarah (UX)</b> wrote: please stop. we have a component library. it is right there.
                </div>
                <div className="guest">
                    <b>me</b> wrote: testing does this work
                </div>
                <div className="guest">
                    <b>me</b> wrote: ok it works
                </div>
                <div className="guest">
                    <b>tim berners-lee</b> wrote: i am so sorry
                </div>
            </div>

            <div className="footer">
                <p>
                    © 2003-2026 me. all rites reserved. do not steal. <span className="blink">NEW!</span>
                </p>
                <p className="ie">best viewed in Internet Explorer 6 at 800×600</p>
                <p className="small">
                    this page is intentionally like this. the rest of the site is fine-ish, which is why i need help. <a href="/discord">join the discord</a> · <a href="/">go back to the nice part</a>
                </p>
            </div>
        </div>
    );
}
