import { Button, GridNotice } from "frontend";

// What a grid screen shows in place of the grid: a missing grid (the grid
// page), or one the viewer may not edit (the editor). `action` is the way out.
// Copy is the product's own.

export const NotFound = () => (
    <GridNotice
        title="Grid not found"
        body="It may have been deleted, or the link is wrong."
        action={
            <Button render={<a href="/grids" />} variant="outline" className="mt-6">
                Browse grids
            </Button>
        }
    />
);

export const NotYours = () => (
    <GridNotice
        title="You can't edit this grid"
        body="Only its owner can change it. You can still use it as a template."
        action={
            <Button render={<a href="/grids/about-me-i95xd5" />} variant="outline" className="mt-6">
                View the grid
            </Button>
        }
    />
);
