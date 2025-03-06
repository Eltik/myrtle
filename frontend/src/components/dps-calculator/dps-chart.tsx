import React, { useState } from "react";
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from "recharts";
import type { Operator } from "~/types/impl/api/static/operator";
import { Button } from "~/components/ui/button";

interface DPSChartProps {
    operators: Operator[];
    generateChartData: () => Promise<unknown[]>;
}

export const DPSChart: React.FC<DPSChartProps> = ({ operators, generateChartData }) => {
    const [chartData, setChartData] = useState<unknown[]>([]);
    const [isLoading, setIsLoading] = useState(false);

    const handleGenerateChart = async () => {
        setIsLoading(true);
        const data = await generateChartData();
        setChartData(data);
        setIsLoading(false);
    };

    const colors = ["#8884d8", "#82ca9d", "#ffc658", "#ff7300", "#a4de6c"];

    return (
        <div className="h-[500px] w-full">
            <Button onClick={handleGenerateChart} disabled={isLoading}>
                {isLoading ? "Generating..." : "Generate Chart"}
            </Button>
            {chartData.length > 0 && (
                <ResponsiveContainer width="100%" height="100%">
                    <LineChart data={chartData}>
                        <CartesianGrid strokeDasharray="6 6" />
                        <XAxis dataKey="defense" />
                        <YAxis />
                        <Tooltip />
                        <Legend />
                        {operators.map((operator, index) => (
                            <Line key={operator.id} type="monotone" dataKey={operator.name} stroke={colors[index % colors.length]} />
                        ))}
                    </LineChart>
                </ResponsiveContainer>
            )}
        </div>
    );
};
