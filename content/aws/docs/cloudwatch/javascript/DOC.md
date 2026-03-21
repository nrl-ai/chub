---
name: cloudwatch
description: "AWS SDK for JavaScript v3 CloudWatch client for custom metrics, metric queries, alarms, and dashboards."
metadata:
  languages: "javascript"
  versions: "3.1006.0"
  revision: 1
  updated-on: "2026-03-11"
  source: maintainer
  tags: "aws,cloudwatch,javascript,nodejs,browser,monitoring,metrics,alarms"
---

# `@aws-sdk/client-cloudwatch`

Use this package for Amazon CloudWatch metrics and alarms in AWS SDK for JavaScript v3. It follows the v3 client-plus-command pattern and is the package to use for publishing custom metrics, querying metric time series, creating alarms, and managing dashboards.

Prefer `CloudWatchClient` plus explicit command imports. The package also exposes an aggregated `CloudWatch` client, but command-based imports are the safer default for smaller bundles and clearer dependency boundaries.

## Install

```bash
npm install @aws-sdk/client-cloudwatch
```

Common companion package:

```bash
npm install @aws-sdk/credential-providers
```

## Initialize the client

```javascript
import { CloudWatchClient } from "@aws-sdk/client-cloudwatch";

const cloudwatch = new CloudWatchClient({
  region: process.env.AWS_REGION ?? "us-east-1",
});
```

## Credentials and Region

- Node.js: the default credential provider chain usually works if AWS access is already configured through environment variables, shared AWS config files, ECS, EC2, or IAM Identity Center.
- Browser runtimes: use an explicit browser-safe credential provider such as Cognito identity; do not ship privileged CloudWatch management credentials to the browser.
- Region is required somewhere. Set it in the client constructor, via `AWS_REGION`, or through shared AWS config.

Typical local setup:

```bash
export AWS_REGION=us-east-1
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

## Core Usage Pattern

The v3 SDK uses `client.send(new Command(input))`.

```javascript
import {
  CloudWatchClient,
  PutMetricDataCommand,
} from "@aws-sdk/client-cloudwatch";

const cloudwatch = new CloudWatchClient({ region: "us-east-1" });

await cloudwatch.send(
  new PutMetricDataCommand({
    Namespace: "MyApp/Orders",
    MetricData: [
      {
        MetricName: "CheckoutLatency",
        Dimensions: [
          { Name: "Environment", Value: "prod" },
          { Name: "Service", Value: "api" },
        ],
        Unit: "Milliseconds",
        Value: 182,
      },
    ],
  }),
);
```

Treat the combination of namespace, metric name, and dimensions as the metric identity you will later query and alarm on.

## Common Operations

### Publish a high-resolution custom metric

Set `StorageResolution: 1` when you need one-second granularity.

```javascript
import {
  CloudWatchClient,
  PutMetricDataCommand,
} from "@aws-sdk/client-cloudwatch";

const cloudwatch = new CloudWatchClient({ region: "us-east-1" });

await cloudwatch.send(
  new PutMetricDataCommand({
    Namespace: "MyApp/Workers",
    MetricData: [
      {
        MetricName: "QueueLag",
        Dimensions: [{ Name: "QueueName", Value: "jobs" }],
        Unit: "Count",
        Value: 3,
        StorageResolution: 1,
      },
    ],
  }),
);
```

### Query recent metric data

Use `GetMetricData` when you need one or more metric series over a time window.

```javascript
import {
  CloudWatchClient,
  GetMetricDataCommand,
} from "@aws-sdk/client-cloudwatch";

const cloudwatch = new CloudWatchClient({ region: "us-east-1" });
const endTime = new Date();
const startTime = new Date(endTime.getTime() - 15 * 60 * 1000);

const response = await cloudwatch.send(
  new GetMetricDataCommand({
    StartTime: startTime,
    EndTime: endTime,
    MetricDataQueries: [
      {
        Id: "checkoutLatencyP95",
        MetricStat: {
          Metric: {
            Namespace: "MyApp/Orders",
            MetricName: "CheckoutLatency",
            Dimensions: [
              { Name: "Environment", Value: "prod" },
              { Name: "Service", Value: "api" },
            ],
          },
          Period: 60,
          Stat: "p95",
          Unit: "Milliseconds",
        },
        ReturnData: true,
      },
    ],
  }),
);

for (const result of response.MetricDataResults ?? []) {
  console.log(result.Id, result.Timestamps, result.Values);
}
```

### Create or update an alarm

`PutMetricAlarm` creates the alarm if it does not exist and updates it when the alarm name already exists.

```javascript
import {
  CloudWatchClient,
  PutMetricAlarmCommand,
} from "@aws-sdk/client-cloudwatch";

const cloudwatch = new CloudWatchClient({ region: "us-east-1" });

await cloudwatch.send(
  new PutMetricAlarmCommand({
    AlarmName: "checkout-latency-p95-too-high",
    Namespace: "MyApp/Orders",
    MetricName: "CheckoutLatency",
    Dimensions: [
      { Name: "Environment", Value: "prod" },
      { Name: "Service", Value: "api" },
    ],
    Statistic: "Average",
    Period: 60,
    EvaluationPeriods: 5,
    Threshold: 500,
    ComparisonOperator: "GreaterThanThreshold",
    TreatMissingData: "notBreaching",
    AlarmDescription: "Average checkout latency is above 500 ms",
  }),
);
```

## CloudWatch-Specific Gotchas

- This package is for CloudWatch metrics, alarms, and dashboards. It is not the log ingestion and query client; use `@aws-sdk/client-cloudwatch-logs` for log groups, log streams, and Logs Insights APIs.
- Custom metric reads and alarms only match the exact metric identity you publish. Keep namespace, metric name, dimension names, and dimension values consistent.
- Use `GetMetricData` for multi-series queries or metric math. `GetMetricStatistics` is still available, but `GetMetricData` is usually the better default for new application code.
- High-resolution datapoints require `StorageResolution: 1`, and alarms must use periods that are compatible with the metric resolution you publish.
- Recently published datapoints can take a short time to become visible in reads and alarm evaluations. Do not assume a write is immediately queryable.
- Alarm actions such as SNS topics or Auto Scaling policies are separate resources. Creating an alarm does not create those downstream integrations for you.
- Do not deep-import internals from package build directories.

## When To Reach For Other Packages

- `@aws-sdk/client-cloudwatch-logs`: log groups, log streams, subscription filters, and Logs Insights queries.
- `@aws-sdk/credential-providers`: Cognito, STS assume-role flows, shared config helpers, and other credential providers.
- `@aws-sdk/client-sns`: create or manage SNS topics that alarm actions publish to.

## Common CloudWatch operations

### List metrics in a namespace

`ListMetrics` helps you discover which metric names and dimensions currently exist.

```javascript
import {
  CloudWatchClient,
  ListMetricsCommand,
} from "@aws-sdk/client-cloudwatch";

const cloudwatch = new CloudWatchClient({ region: "us-east-1" });

let nextToken;

do {
  const response = await cloudwatch.send(
    new ListMetricsCommand({
      Namespace: "MyApp/Orders",
      NextToken: nextToken,
    }),
  );

  for (const metric of response.Metrics ?? []) {
    console.log(metric.MetricName, metric.Dimensions);
  }

  nextToken = response.NextToken;
} while (nextToken);
```

### Describe alarms by name prefix

```javascript
import {
  CloudWatchClient,
  DescribeAlarmsCommand,
} from "@aws-sdk/client-cloudwatch";

const cloudwatch = new CloudWatchClient({ region: "us-east-1" });

let nextToken;

do {
  const response = await cloudwatch.send(
    new DescribeAlarmsCommand({
      AlarmNamePrefix: "checkout-",
      NextToken: nextToken,
    }),
  );

  for (const alarm of response.MetricAlarms ?? []) {
    console.log(alarm.AlarmName, alarm.StateValue, alarm.StateUpdatedTimestamp);
  }

  nextToken = response.NextToken;
} while (nextToken);
```

### Delete alarms

```javascript
import {
  CloudWatchClient,
  DeleteAlarmsCommand,
} from "@aws-sdk/client-cloudwatch";

const cloudwatch = new CloudWatchClient({ region: "us-east-1" });

await cloudwatch.send(
  new DeleteAlarmsCommand({
    AlarmNames: ["checkout-latency-p95-too-high"],
  }),
);
```

### Create or replace a dashboard

`DashboardBody` is a JSON string, so build the object in JavaScript first and then serialize it.

```javascript
import {
  CloudWatchClient,
  PutDashboardCommand,
} from "@aws-sdk/client-cloudwatch";

const cloudwatch = new CloudWatchClient({ region: "us-east-1" });

const dashboardBody = {
  widgets: [
    {
      type: "metric",
      x: 0,
      y: 0,
      width: 12,
      height: 6,
      properties: {
        title: "Checkout latency",
        region: "us-east-1",
        stat: "Average",
        period: 60,
        metrics: [
          [
            "MyApp/Orders",
            "CheckoutLatency",
            "Environment",
            "prod",
          ],
        ],
      },
    },
  ],
};

await cloudwatch.send(
  new PutDashboardCommand({
    DashboardName: "myapp-orders",
    DashboardBody: JSON.stringify(dashboardBody),
  }),
);
```

### Notes

- For custom metrics, discovery via `ListMetrics` is useful during setup, but production code should usually know the namespace and dimensions it writes.
- Alarm names are regional and account-scoped identifiers; treat them as stable resource names.
- Dashboards are replace-on-write documents. Keep the source dashboard JSON in code or configuration if you want reproducible updates.
