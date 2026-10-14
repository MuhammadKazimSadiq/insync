export interface RulePredicate {
  property: string;
  operator: 'equals' | 'notEquals' | 'exists' | 'notExists' | 'contains' | 'in';
  value?: any;
}

export type AttachmentMode = 'markdown-only' | 'explicit' | 'referenced';

export interface ProfileConfig {
  id: string;
  name: string;
  sourceVault: string;
  destinationVault: string;
  scopes: string[];
  excludes: string[];
  rules?: {
    all?: RulePredicate[];
    any?: RulePredicate[];
  } | RulePredicate[] | null;
  attachments?: {
    mode: AttachmentMode;
  } | null;
  autoSync?: boolean;
}

export type PlanAction =
  | { Create: { relative_path: string } }
  | { Update: { relative_path: string } }
  | { Archive: { relative_path: string } }
  | { Skip: { relative_path: string } }
  | { Conflict: { relative_path: string; reason: string } }
  | { Warn: { relative_path?: string; reason: string } };

export interface Plan {
  profile_id: string;
  source_vault: string;
  destination_vault: string;
  actions: PlanAction[];
}

export interface RunReport {
  profile_id: string;
  created_count: number;
  updated_count: number;
  archived_count: number;
  skipped_count: number;
  conflict_count: number;
  warn_count: number;
}

export interface HistoryEntry {
  id: string;
  timestamp: string;
  profileId: string;
  created: number;
  updated: number;
  archived: number;
  skipped: number;
  conflicts: number;
  warnings: number;
  durationMs: number;
}
