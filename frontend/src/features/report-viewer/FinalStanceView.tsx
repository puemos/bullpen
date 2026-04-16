import { ArrowsClockwise, CheckCircle, Eye } from '@phosphor-icons/react';
import { Badge } from '@/components/ui/badge';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import type { FinalStance } from '@/types';
import { ConfidenceBadge, getStanceClasses } from './badge-styles';

interface FinalStanceViewProps {
  stance: FinalStance;
}

export function FinalStanceView({ stance }: FinalStanceViewProps) {
  return (
    <Card>
      <CardHeader className="gap-3">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <CardTitle>Final Conclusion</CardTitle>
            <CardDescription>{stance.horizon}</CardDescription>
          </div>
          <div className="flex flex-wrap gap-2">
            <Badge className={getStanceClasses(stance.stance)}>
              {stance.stance.replace(/_/g, ' ')}
            </Badge>
            <ConfidenceBadge confidence={stance.confidence} />
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        <p className="text-sm leading-relaxed">{stance.summary}</p>
        <div className="grid gap-6 text-sm lg:grid-cols-3">
          <StanceList
            title="Key Reasons"
            items={stance.key_reasons}
            icon={<CheckCircle size={16} weight="bold" />}
            borderClass="border-foreground"
          />
          <StanceList
            title="Watch Items"
            items={stance.watch_items}
            icon={<Eye size={16} weight="bold" />}
            borderClass="border-amber-500/50"
          />
          <StanceList
            title="What Would Change"
            items={stance.what_would_change}
            icon={<ArrowsClockwise size={16} weight="bold" />}
            borderClass="border-muted-foreground/30"
          />
        </div>
      </CardContent>
    </Card>
  );
}

function StanceList({
  title,
  items,
  icon,
  borderClass,
}: {
  title: string;
  items: string[];
  icon: React.ReactNode;
  borderClass: string;
}) {
  if (items.length === 0) return null;

  return (
    <div className="space-y-3">
      <h4 className="flex items-center gap-1.5 font-semibold">
        {icon}
        {title}
      </h4>
      <ul className="space-y-2 text-muted-foreground">
        {items.map(item => (
          <li key={item} className={`border-l-2 ${borderClass} pl-3`}>
            {item}
          </li>
        ))}
      </ul>
    </div>
  );
}
