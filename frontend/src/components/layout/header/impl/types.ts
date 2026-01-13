export type DropdownItem = {
    label: string;
    href: string;
    description: string;
    external?: boolean;
};

export type NavItem = {
    label: string | React.ReactNode;
    href: string;
    dropdown?: DropdownItem[];
};
