import {
  addRunProgress,
  appendRunProgress,
  setRunPlan,
} from '@/store';
import type {
  ProgressEventPayload,
  ProgressItem,
  ToolCallCompleteData,
  ToolCallStartedData,
} from '@/types';

export type ToolTimelineStatus = 'running' | 'completed' | 'failed';

export type TimelineBlock =
  | {
      type: 'message';
      id: string;
      content: string;
    }
  | {
      type: 'tool';
      id: string;
      title: string;
      toolName: string | null;
      kind: string | null;
      arguments: string | null;
      result: string | null;
      status: ToolTimelineStatus;
    }
  | {
      type: 'error';
      id: string;
      content: string;
    }
  | {
      type: 'system';
      id: string;
      content: string;
    };

export function handleProgressEvent(payload: ProgressEventPayload, runId: string) {
  switch (payload.event) {
    case 'MessageDelta':
      appendRunProgress(runId, 'agent_message', payload.data.delta);
      break;
    case 'ThoughtDelta':
      appendRunProgress(runId, 'agent_thought', payload.data.delta);
      break;
    case 'ToolCallStarted':
      addRunProgress(runId, 'tool_call', payload.data.title, payload.data);
      break;
    case 'ToolCallComplete':
      addRunProgress(runId, 'tool_result', `${payload.data.title || 'tool'} ${payload.data.status}`, payload.data);
      break;
    case 'Plan':
      setRunPlan(runId, payload.data.entries);
      addRunProgress(runId, 'plan', 'Plan updated', payload.data);
      break;
    case 'PlanSubmitted':
      addRunProgress(runId, 'submitted', 'Research plan submitted');
      break;
    case 'SourceSubmitted':
      addRunProgress(runId, 'submitted', 'Source submitted');
      break;
    case 'MetricSubmitted':
      addRunProgress(runId, 'submitted', 'Metric submitted');
      break;
    case 'ArtifactSubmitted':
      addRunProgress(runId, 'submitted', 'Structured artifact submitted');
      break;
    case 'BlockSubmitted':
      addRunProgress(runId, 'submitted', 'Analysis block submitted');
      break;
    case 'StanceSubmitted':
      addRunProgress(runId, 'submitted', 'Final stance submitted');
      break;
    case 'Completed':
      addRunProgress(runId, 'completed', 'Analysis complete');
      break;
    case 'Error':
      addRunProgress(runId, 'error', payload.data.message);
      break;
    case 'Log':
      addRunProgress(runId, 'log', payload.data);
      break;
  }
}

export function getTimelineBlocks(progress: ProgressItem[]): TimelineBlock[] {
  const blocks: TimelineBlock[] = [];
  const toolBlockIndexes = new Map<string, number>();
  let currentMessageBlock: Extract<TimelineBlock, { type: 'message' }> | null = null;

  for (const item of progress) {
    if (item.type === 'agent_message' || item.type === 'agent_thought') {
      if (currentMessageBlock) {
        currentMessageBlock.content += item.message;
      } else {
        currentMessageBlock = {
          type: 'message',
          id: item.id,
          content: item.message,
        };
        blocks.push(currentMessageBlock);
      }
      continue;
    }

    currentMessageBlock = null;

    if (item.type === 'tool_call') {
      const data = item.data as ToolCallStartedData | undefined;
      const id = data?.tool_call_id || item.id;
      const block: TimelineBlock = {
        type: 'tool',
        id,
        title: data?.title || item.message,
        toolName: null,
        kind: data?.kind ?? null,
        arguments: null,
        result: null,
        status: 'running',
      };
      toolBlockIndexes.set(id, blocks.length);
      blocks.push(block);
      continue;
    }

    if (item.type === 'tool_result') {
      const data = item.data as ToolCallCompleteData | undefined;
      const id = data?.tool_call_id || item.id;
      const index = toolBlockIndexes.get(id);
      const nextStatus = normalizeToolStatus(data?.status);

      if (index === undefined) {
        const block: TimelineBlock = {
          type: 'tool',
          id,
          title: data?.title || item.message,
          toolName: toolNameFromInput(data?.raw_input),
          kind: null,
          arguments: serializeToolPayload(data?.raw_input),
          result: serializeToolPayload(data?.raw_output),
          status: nextStatus,
        };
        toolBlockIndexes.set(id, blocks.length);
        blocks.push(block);
      } else {
        const block = blocks[index];
        if (block.type === 'tool') {
          block.title = data?.title || block.title;
          block.toolName = toolNameFromInput(data?.raw_input) || block.toolName;
          block.arguments = serializeToolPayload(data?.raw_input);
          block.result = serializeToolPayload(data?.raw_output);
          block.status = nextStatus;
        }
      }
      continue;
    }

    if (item.type === 'error') {
      blocks.push({ type: 'error', id: item.id, content: item.message });
      continue;
    }

    blocks.push({ type: 'system', id: item.id, content: item.message });
  }

  return blocks;
}

function normalizeToolStatus(status: string | undefined): ToolTimelineStatus {
  if (status === 'failed') return 'failed';
  if (status === 'running') return 'running';
  return 'completed';
}

function serializeToolPayload(payload: unknown): string | null {
  if (payload === null || payload === undefined) return null;
  if (typeof payload === 'string') return payload;
  return JSON.stringify(payload);
}

function toolNameFromInput(input: unknown): string | null {
  if (!input || typeof input !== 'object') return null;
  const candidate = input as Record<string, unknown>;
  if (typeof candidate.name === 'string') return candidate.name;
  if (typeof candidate.tool_name === 'string') return candidate.tool_name;
  if (typeof candidate.toolName === 'string') return candidate.toolName;
  return null;
}
