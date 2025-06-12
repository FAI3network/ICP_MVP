import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  ReferenceArea,
  ReferenceLine,
  Legend,
} from "recharts";

import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "../ui";

export default function LineChartchart({
  dataKey,
  label,
  color,
  chartData,
  unfairRange,
  maxVal,
  minVal,
  map = true,
  iterable = true
}: any) {
  //   console.log(label, ": ", Math.min(unfairRange[0], minVal));

  const dataset = []; // [{timestamp: "2021-10-01", <dataKey>: 32}, ...]
  const variableNames: string[] = [];
  const formattedDataKey = dataKey.split(".")

  for (let i = 0; i < chartData.length; i++) {
    const result: { [key: string]: any } = { timestamp: chartData[i].timestamp };

    if (map) {
      for (const variable of chartData[i][dataKey] as { variable_name: string, value: number }[]) {
        result[variable.variable_name] = variable.value;
        if (!variableNames.includes(variable.variable_name)) {
          variableNames.push(variable.variable_name);
        }
      }
    } else {
      const varName = formattedDataKey[formattedDataKey.length - 1];

      // if dataKey is a nested key like "metrics.overall_accuracy" we need get use the formattedDataKey to access the value
      if (formattedDataKey.length > 1) {
        let value = chartData[i];
        for (const key of formattedDataKey) {
          value = value[key];
        }

        const newResult = iterable ? value[0] : value
        result[varName] = newResult ?? 0; // Fallback to 0 if value is undefined
      }
      // if dataKey is a direct key
      else {
        // result["data"] = chartData[i][dataKey];
        const newResult = iterable ? chartData[i][dataKey][0] : chartData[i][dataKey];
        result[varName] = newResult ?? 0; // Fallback to 0 if value is undefined
      }

      if (!variableNames.includes(varName)) {
        variableNames.push(varName);
        // console.log("Variable Name:", varName, "Value:", result[varName]);
      }
    }
    
    dataset.push(result);
    // console.log("Dataset Entry:", result);
  }

  return (
    <ChartContainer
      config={{ [dataKey]: { label, color } }}
      className="min-h-[200px] w-full mb-8 "
    >
      <AreaChart
        width={500}
        height={400}
        data={dataset}
        margin={{
          top: 30,
          right: 30,
          left: 0,
          bottom: 0,
        }}
      >
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis
          dataKey="timestamp"
          tickLine={false}
          tickMargin={10}
          axisLine={false}
          // tickFormatter={(value) => value.slice(5, 10)} // Formats to show only MM-DD
          tickFormatter={(_, index) => `v${index + 1}`}
        />
        <YAxis
          domain={[
            Math.min(unfairRange[0], minVal),
            Math.max(unfairRange[1], maxVal),
          ]}
          tickFormatter={(value) => value.toFixed(2)}
        />
        <ChartTooltip content={<ChartTooltipContent />} />
        <Legend />
        {/* <Area type="monotone" dataKey={dataKey} stroke={color} fill={color} /> */}

        {variableNames.map((variableName, index) => {
          const colorWithIndex = `#${color
            .split("#")[1]
            .split("")
            .map((char: string, i: any) => {
              const dimFactor = Math.max(0, parseInt(char, 16) - parseInt(((index) * 3).toString(), 16));
              return dimFactor.toString(16);
            })
            .join("")}`;
          return (
            <Area
              key={index}
              type="monotone"
              dataKey={variableName}
              stroke={colorWithIndex}
              fill={colorWithIndex}
            />
          );
        })}

        <ReferenceLine
          y={unfairRange[0]}
          stroke="red"
          strokeDasharray="3 3"
          label={{
            value: `Unfairness Limit (${unfairRange[0]})`,
            position: "insideBottomRight",
          }}
        />
        <ReferenceLine
          y={unfairRange[1]}
          stroke="red"
          strokeDasharray="3 3"
          label={{
            value: `Unfairness Limit (${unfairRange[1]})`,
            position: "insideBottomRight",
          }}
        />
      </AreaChart>
    </ChartContainer>
  );
}
