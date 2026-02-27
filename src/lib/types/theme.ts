export type ThemeId = 'cyberpunk' | 'default';

export type ColorMode = 'dark' | 'light' | 'auto';

export interface Theme {
    id: ThemeId;
    name: string;
    primary: string;
    secondary: string;
    glow: string;
}
