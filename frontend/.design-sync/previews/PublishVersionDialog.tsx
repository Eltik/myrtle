import { PublishVersionDialog } from "frontend";
import type { ReactNode } from "react";

const noop = () => {};

const Stage = ({ children }: { children: ReactNode }) => <div className="min-h-[520px]">{children}</div>;

export const FirstVersion = () => (
    <Stage>
        <PublishVersionDialog open publishing={false} latestVersion={null} nextVersion={1} publishError={null} onClose={noop} onPublish={noop} />
    </Stage>
);

export const WithVersionHistory = () => (
    <Stage>
        <PublishVersionDialog open publishing={false} latestVersion={7} nextVersion={8} publishError={null} onClose={noop} onPublish={noop} />
    </Stage>
);

export const Publishing = () => (
    <Stage>
        <PublishVersionDialog open publishing latestVersion={7} nextVersion={8} publishError={null} onClose={noop} onPublish={noop} />
    </Stage>
);

export const PublishFailed = () => (
    <Stage>
        <PublishVersionDialog open publishing={false} latestVersion={7} nextVersion={8} publishError="Rate limited — you can publish one version every 5 minutes. Try again in 3:12." onClose={noop} onPublish={noop} />
    </Stage>
);
