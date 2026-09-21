import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { GameText } from "./GameText";

// `globals: false`, so RTL's auto-cleanup is not installed.
afterEach(cleanup);

const out = () => screen.getByTestId("out");

describe("GameText", () => {
    it("renders a Unity inline tag as an element, not as literal text", () => {
        render(
            <p data-testid="out">
                <GameText text="A <i>clear</i> battle plan." />
            </p>,
        );
        expect(out().textContent).toBe("A clear battle plan.");
        expect(out().innerHTML).toBe("A <i>clear</i> battle plan.");
    });

    it("highlights a query inside a styled span", () => {
        render(
            <p data-testid="out">
                <GameText text="A <i>clear</i> battle plan." highlight="CLEAR" />
            </p>,
        );
        expect(out().textContent).toBe("A clear battle plan.");
        expect(out().querySelector("i > mark")?.textContent).toBe("clear");
    });

    it("highlights a query that straddles a tag boundary", () => {
        render(
            <p data-testid="out">
                <GameText text="A <i>clear</i> battle plan." highlight="clear battle" />
            </p>,
        );
        const marks = Array.from(out().querySelectorAll("mark")).map((m) => m.textContent);
        expect(marks).toEqual(["clear", " battle"]);
        expect(out().textContent).toBe("A clear battle plan.");
    });

    it("highlights every occurrence", () => {
        render(
            <p data-testid="out">
                <GameText text="go, <b>go</b>, go" highlight="go" />
            </p>,
        );
        expect(out().querySelectorAll("mark").length).toBe(3);
        expect(out().querySelector("b > mark")?.textContent).toBe("go");
    });

    it("colours a game span from the palette, or drops it when plain", () => {
        render(
            <p data-testid="out">
                <GameText text="<@ba.vup>+10%</> x" palette="game" />
            </p>,
        );
        const span = out().querySelector("span");
        expect(span?.textContent).toBe("+10%");
        expect(span?.style.color).toBe("rgb(100, 149, 237)");
        cleanup();
        render(
            <p data-testid="out">
                <GameText text="<@ba.vup>+10%</> x" />
            </p>,
        );
        expect(out().innerHTML).toBe("+10% x");
    });

    it("keeps an angle-bracketed item name visible", () => {
        render(
            <p data-testid="out">
                <GameText text="Deploy <@lv.item><Roadblock></> here" />
            </p>,
        );
        expect(out().textContent).toBe("Deploy <Roadblock> here");
    });
});
