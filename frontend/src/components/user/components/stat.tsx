function Stat({ label, value }: { label: string; value: number }) {
    return (
        <div className="text-center">
            <div className="text-2xl font-bold">{value.toLocaleString()}</div>
            <div className="text-sm text-muted-foreground">{label}</div>
        </div>
    );
}

export default Stat;
