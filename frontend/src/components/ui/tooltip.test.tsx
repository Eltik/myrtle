import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { Tooltip, TooltipPopup, TooltipTrigger } from "./tooltip";

function mountOpen() {
    return render(
        <Tooltip open>
            <TooltipTrigger render={<button type="button" />}>trigger</TooltipTrigger>
            <TooltipPopup>hint</TooltipPopup>
        </Tooltip>,
    );
}

afterEach(() => {
    cleanup();
    Object.defineProperty(document, "fullscreenElement", { configurable: true, value: null });
    document.body.replaceChildren();
});

describe("TooltipPopup", () => {
    it("portals to the body when nothing is fullscreen", () => {
        mountOpen();
        expect(document.body.contains(screen.getByText("hint"))).toBe(true);
    });

    it("joins the fullscreen element, where a body portal would be hidden", () => {
        const stage = document.createElement("div");
        document.body.append(stage);
        Object.defineProperty(document, "fullscreenElement", { configurable: true, value: stage });
        mountOpen();
        expect(stage.contains(screen.getByText("hint"))).toBe(true);
    });
});
