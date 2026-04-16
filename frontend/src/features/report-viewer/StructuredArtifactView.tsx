import { Badge } from '@/components/ui/badge';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import type { ArtifactPoint, StructuredArtifact } from '@/types';

interface StructuredArtifactViewProps {
  artifact: StructuredArtifact;
}

export function StructuredArtifactView({ artifact }: StructuredArtifactViewProps) {
  const chartPoints = artifact.series.flatMap(series =>
    series.points.map(point => ({ ...point, series: series.label }))
  );
  const showBarChart = artifact.kind === 'bar_chart' && chartPoints.length > 0;
  const showLineChart = artifact.kind === 'line_chart' && chartPoints.length > 1;

  return (
    <Card>
      <CardHeader className="gap-3">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div className="space-y-1">
            <CardTitle>{artifact.title}</CardTitle>
            <CardDescription>{artifact.summary}</CardDescription>
          </div>
          <div className="flex flex-wrap gap-2">
            <Badge variant="secondary">{formatKind(artifact.kind)}</Badge>
            {artifact.evidence_ids.length > 0 && (
              <Badge variant="outline">{artifact.evidence_ids.length} sources</Badge>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-5">
        {showBarChart && <BarChart points={chartPoints} />}
        {showLineChart && <LineChart points={chartPoints} />}
        {artifact.columns.length > 0 && artifact.rows.length > 0 && (
          <ArtifactTable artifact={artifact} />
        )}
      </CardContent>
    </Card>
  );
}

function ArtifactTable({ artifact }: { artifact: StructuredArtifact }) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          {artifact.columns.map(column => (
            <TableHead key={column.key}>
              {column.label}
              {column.unit && (
                <span className="ml-1 text-xs font-normal text-muted-foreground">
                  ({column.unit})
                </span>
              )}
            </TableHead>
          ))}
        </TableRow>
      </TableHeader>
      <TableBody>
        {artifact.rows.map((row, index) => (
          <TableRow key={index}>
            {artifact.columns.map(column => (
              <TableCell key={column.key} className="max-w-[280px]">
                {formatValue(row[column.key])}
              </TableCell>
            ))}
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}

function BarChart({ points }: { points: Array<ArtifactPoint & { series: string }> }) {
  const max = Math.max(...points.map(point => Math.abs(point.value)), 1);

  return (
    <div className="space-y-3">
      {points.map((point, index) => (
        <div
          key={`${point.series}-${point.label}-${index}`}
          className="grid gap-2 sm:grid-cols-[minmax(120px,220px)_1fr_auto] sm:items-center"
        >
          <div className="truncate text-sm font-medium">{point.label}</div>
          <div className="h-2 overflow-hidden rounded-md bg-muted">
            <div
              className="h-full rounded-md bg-primary"
              style={{ width: `${Math.max(4, (Math.abs(point.value) / max) * 100)}%` }}
            />
          </div>
          <div className="font-mono text-sm text-muted-foreground">
            {formatNumber(point.value)}
          </div>
        </div>
      ))}
    </div>
  );
}

function LineChart({ points }: { points: Array<ArtifactPoint & { series: string }> }) {
  const width = 640;
  const height = 180;
  const padding = 20;
  const values = points.map(point => point.value);
  const min = Math.min(...values);
  const max = Math.max(...values);
  const span = max - min || 1;
  const coords = points.map((point, index) => {
    const x =
      points.length === 1
        ? width / 2
        : padding + (index / (points.length - 1)) * (width - padding * 2);
    const y = height - padding - ((point.value - min) / span) * (height - padding * 2);
    return `${x},${y}`;
  });

  return (
    <div className="space-y-3">
      <div className="rounded-md border bg-muted/20 p-3">
        <svg viewBox={`0 0 ${width} ${height}`} role="img" className="h-48 w-full">
          <polyline
            fill="none"
            stroke="currentColor"
            strokeWidth="3"
            points={coords.join(' ')}
            className="text-primary"
          />
          {coords.map((coord, index) => {
            const [cx, cy] = coord.split(',').map(Number);
            return (
              <circle
                key={`${points[index].label}-${index}`}
                cx={cx}
                cy={cy}
                r="4"
                fill="currentColor"
                className="text-primary"
              />
            );
          })}
        </svg>
      </div>
      <div className="flex flex-wrap gap-2 text-xs text-muted-foreground">
        {points.map((point, index) => (
          <span key={`${point.label}-${index}`}>
            {point.label}: {formatNumber(point.value)}
          </span>
        ))}
      </div>
    </div>
  );
}

function formatKind(kind: string) {
  return kind.replace(/_/g, ' ');
}

function formatValue(value: unknown): string {
  if (value === null || value === undefined || value === '') return '—';
  if (typeof value === 'number') return formatNumber(value);
  if (typeof value === 'string') return value;
  if (typeof value === 'boolean') return value ? 'yes' : 'no';
  if (Array.isArray(value)) return value.map(formatValue).join(', ');
  return JSON.stringify(value);
}

function formatNumber(value: number): string {
  return new Intl.NumberFormat(undefined, {
    maximumFractionDigits: Math.abs(value) < 10 ? 2 : 1,
  }).format(value);
}
