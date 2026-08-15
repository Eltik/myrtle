import { LegalContainer, SectionHeading } from "frontend";
import { Database, KeyRound, Lock, ScaleIcon, UserCog } from "lucide-react";

// The `<h2>` block that opens every clause of the privacy / terms documents: a
// rounded primary-tinted icon tile beside the title, with an optional one-line
// subtitle. `PrivacyPage` uses the icon + subtitle form throughout; `TermsPage`
// mostly uses bare titles.

const P = ({ children }: { children: React.ReactNode }) => <p className="m-0 mb-4 font-sans text-[16px] text-foreground leading-[1.75] last:mb-0">{children}</p>;

export const WithSubtitle = () => (
    <div className="max-w-3xl">
        <SectionHeading icon={<KeyRound className="size-5" strokeWidth={1.8} />} title="Yostar Account Integration" subtitle="How we sync your Arknights game data" />
        <P>Myrtle uses Yostar's official OAuth flow. You enter your email, Yostar mails you a verification code, and we exchange that code for a session token — we never see or store your password.</P>
    </div>
);

export const TitleOnly = () => (
    <div className="max-w-3xl">
        <SectionHeading icon={<ScaleIcon className="size-5" strokeWidth={1.8} />} title="Limitation of Liability" />
        <P>myrtle.moe is provided as-is. Game data is reproduced for reference and may lag behind the live servers after a patch.</P>
    </div>
);

export const InDocument = () => (
    <LegalContainer>
        <section className="mb-10">
            <SectionHeading icon={<Database className="size-5" strokeWidth={1.8} />} title="What We Collect" subtitle="Roster, base layout and pull history — only for accounts you link" />
            <P>Linking an account mirrors the payload the game client already receives: owned operators with promotion, level, mastery and module state, your base layout, and the gacha history the server still retains.</P>
        </section>
        <section className="mb-10">
            <SectionHeading icon={<Lock className="size-5" strokeWidth={1.8} />} title="How We Store It" subtitle="Encrypted at rest, never sold, deletable on request" />
            <P>Synced data lives in a Postgres instance in the EU, encrypted at rest. It is never sold or shared with advertisers, and deleting your profile removes every synced row within 24 hours.</P>
        </section>
        <section className="mb-10">
            <SectionHeading icon={<UserCog className="size-5" strokeWidth={1.8} />} title="Your Controls" subtitle="Profile visibility, leaderboard opt-in, gacha sharing" />
            <P>Every sharing switch is off until you turn it on. A private profile is excluded from search, the leaderboard and the community gacha statistics.</P>
        </section>
    </LegalContainer>
);
