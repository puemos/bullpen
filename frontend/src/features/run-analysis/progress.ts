import {
  addProgress,
  appendProgress,
  setState,
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

export function handleProgressEvent(payload: ProgressEventPayload) {
  switch (payload.event) {
    case 'MessageDelta':
      appendProgress('agent_message', payload.data.delta);
      break;
    case 'ThoughtDelta':
      appendProgress('agent_thought', payload.data.delta);
      break;
    case 'ToolCallStarted':
      addProgress('tool_call', payload.data.title, payload.data);
      break;
    case 'ToolCallComplete':
      addProgress('tool_result', `${payload.data.title || 'tool'} ${payload.data.status}`, payload.data);
      break;
    case 'Plan':
      setState({ plan: payload.data.entries });
      addProgress('plan', 'Plan updated', payload.data);
      break;
    case 'PlanSubmitted':
      addProgress('submitted', 'Research plan submitted');
      break;
    case 'SourceSubmitted':
      addProgress('submitted', 'Source submitted');
      break;
    case 'MetricSubmitted':
      addProgress('submitted', 'Metric submitted');
      break;
    case 'BlockSubmitted':
      addProgress('submitted', 'Analysis block submitted');
      break;
    case 'StanceSubmitted':
      addProgress('submitted', 'Final stance submitted');
      break;
    case 'Completed':
      addProgress('completed', 'Analysis complete');
      break;
    case 'Error':
      addProgress('error', payload.data.message);
      break;
    case 'Log':
      addProgress('log', payload.data);
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
        toolName: data?.kind || null,
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
