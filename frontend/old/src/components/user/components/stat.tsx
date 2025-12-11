function Stat({ label, value }: { label: string; value: number }) {
    return (
        <div className="text-center">
            <div className="font-bold text-2xl">{value.toLocaleString()}</div>
            <div className="text-muted-foreground text-sm">{label}</div>
        </div>
    );
}

export default Stat;
