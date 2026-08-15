import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from "frontend";
import { ChevronLeftIcon } from "lucide-react";

export const Default = () => (
    <Breadcrumb>
        <BreadcrumbList>
            <BreadcrumbItem>
                <BreadcrumbLink href="/operators">Operators</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbLink href="/operators?class=sniper">Sniper</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbPage className="font-medium">Exusiai</BreadcrumbPage>
            </BreadcrumbItem>
        </BreadcrumbList>
    </Breadcrumb>
);

export const WithRenderProp = () => (
    <Breadcrumb>
        <BreadcrumbList>
            <BreadcrumbItem>
                {/* `render` swaps the element — in the app this is a TanStack <Link to="/operators" />. */}
                <BreadcrumbLink render={<a className="inline-flex items-center gap-1" href="/operators" />}>
                    <ChevronLeftIcon className="size-3.5" />
                    Back to operators
                </BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbPage className="font-medium">Mlynar</BreadcrumbPage>
            </BreadcrumbItem>
        </BreadcrumbList>
    </Breadcrumb>
);

export const EmphasisedTrail = () => (
    <Breadcrumb>
        <BreadcrumbList>
            <BreadcrumbItem>
                <BreadcrumbLink className="font-medium text-foreground" href="/">
                    Myrtle
                </BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbLink href="/tools">Tools</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbLink href="/tools/recruitment">Recruitment calculator</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbPage className="font-medium">Results</BreadcrumbPage>
            </BreadcrumbItem>
        </BreadcrumbList>
    </Breadcrumb>
);
