import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import * as commands from "./commands";

export const queryKeys = {
  agents: ["agents"] as const,
  analyses: ["analyses"] as const,
  analysis: (id: string) => ["analyses", id] as const,
  report: (id: string, runId?: string) => ["analyses", id, "report", runId ?? "active"] as const,
  portfolios: ["portfolios"] as const,
  portfolio: (id: string) => ["portfolios", id] as const,
  settings: ["settings"] as const,
  sources: ["sources"] as const,
  priceHistory: (symbol: string, market: string | null) =>
    ["priceHistory", symbol, market ?? "default"] as const,
};

export function useAgents() {
  return useQuery({
    queryKey: queryKeys.agents,
    queryFn: commands.getAgents,
  });
}

export function useAnalyses() {
  return useQuery({
    queryKey: queryKeys.analyses,
    queryFn: commands.getAllAnalyses,
  });
}

export function useAnalysisReport(analysisId: string | null, runId?: string) {
  return useQuery({
    queryKey: queryKeys.report(analysisId!, runId),
    queryFn: () => commands.getAnalysisReport(analysisId!, runId),
    enabled: !!analysisId,
  });
}

export function usePortfolios() {
  return useQuery({
    queryKey: queryKeys.portfolios,
    queryFn: commands.getPortfolios,
  });
}

export function usePortfolioDetail(portfolioId: string | null) {
  return useQuery({
    queryKey: queryKeys.portfolio(portfolioId!),
    queryFn: () => commands.getPortfolioDetail(portfolioId!),
    enabled: !!portfolioId,
  });
}

export function useSettings() {
  return useQuery({
    queryKey: queryKeys.settings,
    queryFn: commands.getSettings,
  });
}

export function useSources() {
  return useQuery({
    queryKey: queryKeys.sources,
    queryFn: commands.listSources,
  });
}

export function usePriceHistory(symbol: string, market: string | null, enabled = true) {
  return useQuery({
    queryKey: queryKeys.priceHistory(symbol, market),
    queryFn: () => commands.getPriceHistory(symbol, market),
    enabled,
    staleTime: 1000 * 60 * 5,
  });
}

export function useDeleteAnalysis() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: commands.deleteAnalysis,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.analyses });
    },
  });
}

export function useDeletePortfolio() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: commands.deletePortfolio,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.portfolios });
    },
  });
}

export function useCreatePortfolio() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ name, baseCurrency }: { name: string; baseCurrency: string }) =>
      commands.createPortfolio(name, baseCurrency),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.portfolios });
    },
  });
}

export function useRenamePortfolio() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ portfolioId, name }: { portfolioId: string; name: string }) =>
      commands.renamePortfolio(portfolioId, name),
    onSuccess: (_, { portfolioId }) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.portfolios });
      queryClient.invalidateQueries({ queryKey: queryKeys.portfolio(portfolioId) });
    },
  });
}

export function useUpdateSettings() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: commands.updateSettings,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.settings });
    },
  });
}

export function useSetSourceKey() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ providerId, key }: { providerId: string; key: string }) =>
      commands.setSourceKey(providerId, key),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.sources });
    },
  });
}

export function useClearSourceKey() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: commands.clearSourceKey,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.sources });
    },
  });
}

export function useTestSourceKey() {
  return useMutation({
    mutationFn: commands.testSourceKey,
  });
}

export function useSetEnabledSources() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: commands.setEnabledSources,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.sources });
      queryClient.invalidateQueries({ queryKey: queryKeys.settings });
    },
  });
}

export function useImportPortfolioCsv() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: commands.importPortfolioCsv,
    onSuccess: (result) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.portfolios });
      queryClient.invalidateQueries({ queryKey: queryKeys.portfolio(result.portfolio_id) });
    },
  });
}
