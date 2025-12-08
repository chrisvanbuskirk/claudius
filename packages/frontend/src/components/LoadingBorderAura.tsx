import { motion, AnimatePresence } from 'framer-motion';

interface LoadingBorderAuraProps {
  isActive: boolean;
}

/**
 * Purple animated border aura that appears around the entire window
 * when research is actively running. Creates a cyberpunk-style glow effect
 * on all 4 edges of the screen.
 */
export function LoadingBorderAura({ isActive }: LoadingBorderAuraProps) {
  return (
    <AnimatePresence>
      {isActive && (
        <>
          {/* Top border */}
          <motion.div
            className="pointer-events-none fixed left-0 right-0 top-0 z-50 h-1"
            initial={{ opacity: 0, scaleX: 0 }}
            animate={{
              opacity: [0.6, 1, 0.6],
              scaleX: 1,
            }}
            exit={{ opacity: 0, scaleX: 0 }}
            transition={{
              scaleX: { duration: 0.3, ease: 'easeOut' },
              opacity: {
                duration: 2,
                repeat: Infinity,
                ease: 'easeInOut'
              }
            }}
            style={{
              background: 'linear-gradient(90deg, transparent, #8b5cf6, #a78bfa, #8b5cf6, transparent)',
              boxShadow: '0 0 20px rgba(139, 92, 246, 0.8), 0 0 40px rgba(139, 92, 246, 0.5)',
            }}
          />

          {/* Bottom border */}
          <motion.div
            className="pointer-events-none fixed bottom-0 left-0 right-0 z-50 h-1"
            initial={{ opacity: 0, scaleX: 0 }}
            animate={{
              opacity: [0.6, 1, 0.6],
              scaleX: 1,
            }}
            exit={{ opacity: 0, scaleX: 0 }}
            transition={{
              scaleX: { duration: 0.3, ease: 'easeOut', delay: 0.1 },
              opacity: {
                duration: 2,
                repeat: Infinity,
                ease: 'easeInOut',
                delay: 0.5
              }
            }}
            style={{
              background: 'linear-gradient(90deg, transparent, #8b5cf6, #a78bfa, #8b5cf6, transparent)',
              boxShadow: '0 0 20px rgba(139, 92, 246, 0.8), 0 0 40px rgba(139, 92, 246, 0.5)',
            }}
          />

          {/* Left border */}
          <motion.div
            className="pointer-events-none fixed bottom-0 left-0 top-0 z-50 w-1"
            initial={{ opacity: 0, scaleY: 0 }}
            animate={{
              opacity: [0.6, 1, 0.6],
              scaleY: 1,
            }}
            exit={{ opacity: 0, scaleY: 0 }}
            transition={{
              scaleY: { duration: 0.3, ease: 'easeOut', delay: 0.05 },
              opacity: {
                duration: 2,
                repeat: Infinity,
                ease: 'easeInOut',
                delay: 0.25
              }
            }}
            style={{
              background: 'linear-gradient(180deg, transparent, #8b5cf6, #a78bfa, #8b5cf6, transparent)',
              boxShadow: '0 0 20px rgba(139, 92, 246, 0.8), 0 0 40px rgba(139, 92, 246, 0.5)',
            }}
          />

          {/* Right border */}
          <motion.div
            className="pointer-events-none fixed bottom-0 right-0 top-0 z-50 w-1"
            initial={{ opacity: 0, scaleY: 0 }}
            animate={{
              opacity: [0.6, 1, 0.6],
              scaleY: 1,
            }}
            exit={{ opacity: 0, scaleY: 0 }}
            transition={{
              scaleY: { duration: 0.3, ease: 'easeOut', delay: 0.15 },
              opacity: {
                duration: 2,
                repeat: Infinity,
                ease: 'easeInOut',
                delay: 0.75
              }
            }}
            style={{
              background: 'linear-gradient(180deg, transparent, #8b5cf6, #a78bfa, #8b5cf6, transparent)',
              boxShadow: '0 0 20px rgba(139, 92, 246, 0.8), 0 0 40px rgba(139, 92, 246, 0.5)',
            }}
          />

          {/* Corner glows for extra polish */}
          {[
            { top: 0, left: 0 },
            { top: 0, right: 0 },
            { bottom: 0, left: 0 },
            { bottom: 0, right: 0 }
          ].map((position, i) => (
            <motion.div
              key={i}
              className="pointer-events-none fixed z-50 h-8 w-8"
              style={{
                ...position,
                background: 'radial-gradient(circle, rgba(139, 92, 246, 0.6), transparent)',
              }}
              initial={{ opacity: 0, scale: 0 }}
              animate={{
                opacity: [0.4, 0.8, 0.4],
                scale: 1,
              }}
              exit={{ opacity: 0, scale: 0 }}
              transition={{
                scale: { duration: 0.3, delay: i * 0.05 },
                opacity: {
                  duration: 2,
                  repeat: Infinity,
                  ease: 'easeInOut',
                  delay: i * 0.25
                }
              }}
            />
          ))}
        </>
      )}
    </AnimatePresence>
  );
}
