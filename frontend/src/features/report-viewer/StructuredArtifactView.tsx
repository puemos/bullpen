import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { cn } from '@/lib/utils';
import type { ArtifactPoint, StructuredArtifact } from '@/types';
import { Eyebrow } from '@/components/ui/editorial';

interface StructuredArtifactViewProps {
  artifact: StructuredArtifact;
  isFirst?: boolean;
}

export function StructuredArtifactView({ artifact, isFirst }: StructuredArtifactViewProps) {
  const chartPoints = artifact.series.flatMap(series =>
    series.points.map(point => ({ ...point, series: series.label })),
  );
  const showBarChart = artifact.kind === 'bar_chart' && chartPoints.length > 0;
  const showLineChart = artifact.kind === 'line_chart' && chartPoints.length > 1;
  const showAreaChart = artifact.kind === 'area_chart' && chartPoints.length > 1;

  return (
    <article
      className={
        isFirst
          ? 'space-y-6 py-8'
          : 'space-y-6 border-t border-border py-8'
      }
    >
      <header className="flex flex-wrap items-baseline justify-between gap-3">
        <div className="space-y-2">
          <Eyebrow>{formatKind(artifact.kind)}</Eyebrow>
          <h3 className="text-[17px] font-semibold leading-snug tracking-tight">
            {artifact.title}
          </h3>
          {artifact.summary && (
            <p className="max-w-[62ch] text-sm leading-relaxed text-muted-foreground">
              {artifact.summary}
            </p>
          )}
        </div>
        {artifact.evidence_ids.length > 0 && (
          <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground">
            {String(artifact.evidence_ids.length).padStart(2, '0')} sources
          </span>
        )}
      </header>
      {showBarChart && <BarChart points={chartPoints} />}
      {showLineChart && <LineChart points={chartPoints} />}
      {showAreaChart && <AreaChart points={chartPoints} artifactId={artifact.id} />}
      {artifact.columns.length > 0 && artifact.rows.length > 0 && (
        <ArtifactTable artifact={artifact} />
      )}
    </article>
  );
}

function ArtifactTable({ artifact }: { artifact: StructuredArtifact }) {
  const columnIsNumeric = artifact.columns.map(column =>
    artifact.rows.every(row => {
      const value = row[column.key];
      return value === null || value === undefined || typeof value === 'number';
    }),
  );

  return (
    <div className="overflow-x-auto">
      <Table className="text-[13px]">
        <TableHeader>
          <TableRow className="border-b border-border">
            {artifact.columns.map((column, colIndex) => (
              <TableHead
                key={column.key}
                className={cn(
                  'px-3 align-top text-[10.5px] uppercase tracking-[0.14em] text-muted-foreground',
                  columnIsNumeric[colIndex]
                    ? 'min-w-[96px] whitespace-nowrap text-right'
                    : 'min-w-[180px] max-w-[420px]',
                )}
              >
                {column.label}
                {column.unit && (
                  <span className="ml-1 normal-case tracking-normal">
                    ({column.unit})
                  </span>
                )}
              </TableHead>
            ))}
          </TableRow>
        </TableHeader>
        <TableBody>
          {artifact.rows.map((row, index) => (
            <TableRow key={index} className="border-b border-border/60">
              {artifact.columns.map((column, colIndex) => {
                const value = row[column.key];
                const numeric = columnIsNumeric[colIndex];
                return (
                  <TableCell
                    key={column.key}
                    className={cn(
                      'px-3 align-top',
                      numeric
                        ? 'min-w-[96px] whitespace-nowrap text-right font-mono tabular-nums'
                        : 'min-w-[180px] max-w-[420px] whitespace-normal leading-[1.55]',
                    )}
                  >
                    {formatValue(value)}
                  </TableCell>
                );
              })}
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}

function BarChart({ points }: { points: Array<ArtifactPoint & { series: string }> }) {
  const max = Math.max(...points.map(point => Math.abs(point.value)), 1);

  return (
    <div className="space-y-2">
      {points.map((point, index) => (
        <div
          key={`${point.series}-${point.label}-${index}`}
          className="grid items-center gap-3 text-[13px] sm:grid-cols-[minmax(140px,220px)_1fr_auto]"
        >
          <div className="truncate text-foreground">{point.label}</div>
          <div className="h-[6px] overflow-hidden bg-border/60">
            <div
              className="h-full bg-foreground"
              style={{ width: `${Math.max(4, (Math.abs(point.value) / max) * 100)}%` }}
            />
          </div>
          <div className="font-mono tabular-nums text-muted-foreground">
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
      <div className="border-y border-border py-3">
        <svg viewBox={`0 0 ${width} ${height}`} role="img" className="h-48 w-full">
          <polyline
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            points={coords.join(' ')}
            className="text-foreground"
          />
          {coords.map((coord, index) => {
            const [cx, cy] = coord.split(',').map(Number);
            return (
              <circle
                key={`${points[index].label}-${index}`}
                cx={cx}
                cy={cy}
                r="2.5"
                fill="currentColor"
                className="text-foreground"
              />
            );
          })}
        </svg>
      </div>
      <div className="flex flex-wrap gap-x-4 gap-y-1 font-mono text-[11px] tabular-nums text-muted-foreground">
        {points.map((point, index) => (
          <span key={`${point.label}-${index}`}>
            {point.label} · {formatNumber(point.value)}
          </span>
        ))}
      </div>
    </div>
  );
}

function AreaChart({
  points,
  artifactId,
}: {
  points: Array<ArtifactPoint & { series: string }>;
  artifactId: string;
}) {
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
    return { x, y };
  });
  const baselineY = height - padding;
  const linePoints = coords.map(c => `${c.x},${c.y}`).join(' ');
  const areaPath = [
    `M${coords[0].x},${baselineY}`,
    ...coords.map(c => `L${c.x},${c.y}`),
    `L${coords[coords.length - 1].x},${baselineY}`,
    'Z',
  ].join(' ');
  const gradientId = `area-gradient-${artifactId}`;

  return (
    <div className="space-y-3">
      <div className="border-y border-border py-3">
        <svg viewBox={`0 0 ${width} ${height}`} role="img" className="h-48 w-full">
          <defs>
            <linearGradient id={gradientId} x1="0" x2="0" y1="0" y2="1">
              <stop offset="0%" stopColor="currentColor" stopOpacity="0.28" />
              <stop offset="100%" stopColor="currentColor" stopOpacity="0.02" />
            </linearGradient>
          </defs>
          <path d={areaPath} fill={`url(#${gradientId})`} className="text-foreground" />
          <polyline
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            points={linePoints}
            className="text-foreground"
          />
          {coords.map((c, index) => (
            <circle
              key={`${points[index].label}-${index}`}
              cx={c.x}
              cy={c.y}
              r="2.5"
              fill="currentColor"
              className="text-foreground"
            />
          ))}
        </svg>
      </div>
      <div className="flex flex-wrap gap-x-4 gap-y-1 font-mono text-[11px] tabular-nums text-muted-foreground">
        {points.map((point, index) => (
          <span key={`${point.label}-${index}`}>
            {point.label} · {formatNumber(point.value)}
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
