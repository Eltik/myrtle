import { RowImage } from "frontend";

// A release row's thumbnail: event art at 16:9, banner art at 5:2.
const API = "https://api.myrtle.moe/api";

export const EventArt = () => (
    <div className="w-40 p-4">
        <RowImage src={`${API}/en/event-image/act51side`} alt="People, A People" onError={() => {}} />
    </div>
);

export const BannerArtWide = () => (
    <div className="w-64 p-4">
        <RowImage src={`${API}/cn/banner-image/DOUBLE_77_0_5`} alt="Rare Operators useful in all kinds of stages" onError={() => {}} wide />
    </div>
);
