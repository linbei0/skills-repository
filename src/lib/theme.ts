import { getCurrentWindow } from '@tauri-apps/api/window'
import type { ResolvedTheme, ThemeMode } from '../types/app'

export const resolveThemeMode = (
  themeMode: ThemeMode,
  systemTheme: 'light' | 'dark',
): ResolvedTheme => {
  if (themeMode === 'system') {
    return systemTheme === 'dark' ? 'skills-dark' : 'skills-light'
  }

  return themeMode === 'dark' ? 'skills-dark' : 'skills-light'
}

export const applyResolvedTheme = (theme: ResolvedTheme) => {
  document.documentElement.setAttribute('data-theme', theme)

  const nativeTheme = theme === 'skills-dark' ? 'dark' : 'light'
  void getCurrentWindow().setTheme(nativeTheme).catch((error) => {
    console.error('Failed to sync native window theme:', error)
  })
}
