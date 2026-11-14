import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Pagination } from "./Pagination";

const href = (page: number) => (page > 1 ? `/operators?page=${page}` : "/operators");

function links(container: HTMLElement): string[] {
    return [...container.querySelectorAll("a[href]")].map((a) => a.getAttribute("href") ?? "");
}

afterEach(cleanup);

describe("Pagination", () => {
    it("links every page it shows when the list gives it page URLs", () => {
        const { container } = render(<Pagination currentPage={2} totalPages={5} onPageChange={() => {}} hrefForPage={href} />);
        const found = new Set(links(container));
        // First and previous lead to page one (the bare path), next to three, last to five.
        for (const want of ["/operators", "/operators?page=3", "/operators?page=5"]) expect(found, want).toContain(want);
        // The page you are on leads nowhere, so it is not a link.
        expect(found).not.toContain("/operators?page=2");
        expect(container.querySelector('[aria-current="page"]')?.tagName).toBe("BUTTON");
    });

    it("keeps the ends as disabled buttons, not links to pages that do not exist", () => {
        const first = render(<Pagination currentPage={1} totalPages={3} onPageChange={() => {}} hrefForPage={href} />);
        expect(links(first.container)).not.toContain("/operators?page=0");
        expect(first.container.querySelectorAll("button[disabled]").length).toBeGreaterThan(0);
        first.unmount();

        const last = render(<Pagination currentPage={3} totalPages={3} onPageChange={() => {}} hrefForPage={href} />);
        expect(links(last.container)).not.toContain("/operators?page=4");
    });

    it("pages in place on a plain click and leaves a modified click to the browser", () => {
        const onPageChange = vi.fn();
        const { container } = render(<Pagination currentPage={2} totalPages={5} onPageChange={onPageChange} hrefForPage={href} />);
        const three = [...container.querySelectorAll("a[href]")].find((a) => a.getAttribute("href") === "/operators?page=3" && a.textContent === "3");
        if (!three) throw new Error("no link to page three");

        // `fireEvent` returns false when the handler cancelled the default (the navigation).
        expect(fireEvent.click(three)).toBe(false);
        expect(onPageChange).toHaveBeenCalledWith(3);

        onPageChange.mockClear();
        expect(fireEvent.click(three, { ctrlKey: true })).toBe(true);
        expect(onPageChange).not.toHaveBeenCalled();
    });

    it("stays all buttons without page URLs", () => {
        const onPageChange = vi.fn();
        const { container } = render(<Pagination currentPage={2} totalPages={5} onPageChange={onPageChange} />);
        expect(links(container)).toEqual([]);
        const three = container.querySelector('button[aria-label="Go to page 3"]');
        if (!three) throw new Error("no button for page three");
        fireEvent.click(three);
        expect(onPageChange).toHaveBeenCalledWith(3);
    });
});
