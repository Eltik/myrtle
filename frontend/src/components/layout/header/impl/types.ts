export type NavItem = {
    label: string | React.ReactNode;
    href: string;
    dropdown?: { label: string; href: string; description: string }[];
};
