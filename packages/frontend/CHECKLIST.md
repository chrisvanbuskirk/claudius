# Development Checklist

Use this checklist when working on the Claudius frontend.

## Initial Setup

- [ ] Node.js 18+ installed
- [ ] Run `npm install` successfully
- [ ] Backend (Tauri) is configured
- [ ] Can run `npm run dev` without errors
- [ ] Browser opens to `http://localhost:5173`

## Before Committing Code

### Code Quality
- [ ] No TypeScript errors (`npm run build` passes)
- [ ] No console errors or warnings
- [ ] All imports are used
- [ ] No commented-out code blocks
- [ ] Code follows existing patterns

### Functionality
- [ ] Feature works as expected
- [ ] Loading states display properly
- [ ] Error states handle gracefully
- [ ] Empty states are helpful
- [ ] All user interactions provide feedback

### UI/UX
- [ ] Works in light mode
- [ ] Works in dark mode
- [ ] Responsive (if applicable)
- [ ] Icons are consistent (Lucide)
- [ ] Text is readable
- [ ] Spacing is consistent
- [ ] Animations are smooth

### Type Safety
- [ ] All props are typed
- [ ] No `any` types used
- [ ] Backend responses are typed
- [ ] Event handlers are typed

## Adding New Features

### New Component
- [ ] Create in `src/components/`
- [ ] Export from component file
- [ ] Define prop interface
- [ ] Add proper TypeScript types
- [ ] Use Tailwind classes
- [ ] Support dark mode

### New Page
- [ ] Create in `src/pages/`
- [ ] Add route in `App.tsx`
- [ ] Add navigation in `Sidebar.tsx`
- [ ] Add page title
- [ ] Handle loading state
- [ ] Handle error state
- [ ] Handle empty state

### New Hook
- [ ] Create in `src/hooks/`
- [ ] Return consistent object structure
- [ ] Include loading state
- [ ] Include error state
- [ ] Type all parameters
- [ ] Type return values
- [ ] Handle errors gracefully

### New Tauri Command
- [ ] Add to appropriate hook in `useTauri.ts`
- [ ] Type parameters
- [ ] Type return value
- [ ] Add error handling
- [ ] Update loading state
- [ ] Test with backend

### New Type
- [ ] Add to `src/types/index.ts`
- [ ] Export the type
- [ ] Match backend model exactly
- [ ] Use proper TypeScript syntax
- [ ] Add JSDoc comments (if needed)

## Testing Checklist

### Manual Testing
- [ ] Happy path works
- [ ] Error cases handled
- [ ] Loading states work
- [ ] Empty states work
- [ ] Feedback is clear
- [ ] Navigation works
- [ ] Data persists correctly

### Cross-Browser (if applicable)
- [ ] Chrome/Edge
- [ ] Firefox
- [ ] Safari

### Performance
- [ ] No memory leaks
- [ ] Fast initial load
- [ ] Smooth interactions
- [ ] Efficient re-renders

## Pre-Release Checklist

### Code
- [ ] All features complete
- [ ] All bugs fixed
- [ ] Code is clean
- [ ] No debug logs
- [ ] No TODOs in code

### Build
- [ ] `npm run build` succeeds
- [ ] Build size is reasonable
- [ ] No warnings in build output
- [ ] Preview works (`npm run preview`)

### Documentation
- [ ] README is up to date
- [ ] New features documented
- [ ] Breaking changes noted
- [ ] Examples added (if needed)

### Integration
- [ ] Works with current backend
- [ ] All Tauri commands work
- [ ] No API mismatches
- [ ] Proper error handling

## Common Issues

### TypeScript Errors
- Check that all types are imported
- Verify type definitions match backend
- Run `npm run build` to see all errors
- Check for missing return types

### Styling Issues
- Verify Tailwind classes are correct
- Check dark mode classes (dark:)
- Inspect element in browser dev tools
- Look for conflicting CSS

### Tauri Command Failures
- Verify command name matches backend
- Check parameter types
- Look at browser console for details
- Verify backend is running

### Build Failures
- Clear `node_modules` and reinstall
- Check for syntax errors
- Verify all imports exist
- Check `package.json` for issues

## Best Practices

### Code Style
- Use functional components
- Use hooks for state and effects
- Keep components small and focused
- Extract reusable logic to hooks
- Use descriptive variable names

### TypeScript
- Always define types explicitly
- Use interfaces for objects
- Use type aliases for unions
- Don't use `any` (use `unknown` if needed)

### React
- Use `useCallback` for functions passed to children
- Use `useMemo` for expensive calculations
- Avoid inline function definitions in JSX
- Clean up effects properly

### Tailwind
- Use utility classes over custom CSS
- Follow mobile-first approach
- Use dark: prefix for dark mode
- Group related utilities together

### Performance
- Avoid unnecessary re-renders
- Use proper keys for lists
- Lazy load heavy components (if needed)
- Optimize images and assets

## Resources

- TypeScript: https://www.typescriptlang.org/
- React: https://react.dev/
- Vite: https://vitejs.dev/
- Tailwind: https://tailwindcss.com/
- Tauri: https://tauri.app/
