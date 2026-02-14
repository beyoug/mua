export type TaskViewNav = 'active' | 'complete' | 'history';

export type BulkTrashPlan =
    | { action: 'enter_selection' }
    | { action: 'select_all' }
    | { action: 'execute'; deleteFile: boolean }
    | {
          action: 'confirm';
          dialog: {
              title: string;
              description: string;
              confirmText: string;
          };
      };

export type SingleTaskRemovalPlan =
    | { action: 'cancel' }
    | { action: 'remove'; deleteFile: boolean }
    | {
          action: 'confirm_remove';
          dialog: {
              title: string;
              description: string;
              confirmText: string;
          };
      };
