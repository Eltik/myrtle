import { CurrencyLabel } from "frontend";

// A currency name with its icon in front, as the pull planner's field labels use it.
export const ResourceLabels = () => (
    <div className="flex flex-col gap-2 p-4 font-medium font-sans text-[12.5px] text-muted-foreground">
        <CurrencyLabel name="orundum">Orundum</CurrencyLabel>
        <CurrencyLabel name="permit">Permits</CurrencyLabel>
        <CurrencyLabel name="tenPermit">Ten-roll permits</CurrencyLabel>
        <CurrencyLabel name="originite">Originite Prime</CurrencyLabel>
    </div>
);

export const ShopCurrencies = () => (
    <div className="flex flex-col gap-2 p-4 font-sans text-[13px] text-foreground">
        <CurrencyLabel name="goldCert">Distinction Certificate</CurrencyLabel>
        <CurrencyLabel name="greenCert">Commendation Certificate</CurrencyLabel>
        <CurrencyLabel name="monthlyCard">Monthly Card</CurrencyLabel>
    </div>
);
