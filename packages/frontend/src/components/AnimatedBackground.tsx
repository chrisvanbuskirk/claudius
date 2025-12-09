import { motion } from 'framer-motion';

/**
 * Animated gradient mesh background with slow-moving purple and blue orbs.
 * Creates a cyberpunk "AI thinking" atmosphere behind glassmorphism cards.
 * Respects prefers-reduced-motion for accessibility.
 */
export function AnimatedBackground() {
  const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  return (
    <div className="fixed inset-0 -z-10 overflow-hidden pointer-events-none" aria-hidden="true">
      {/* Base gradient */}
      <div className="absolute inset-0 bg-gradient-to-br from-gray-900 via-gray-900 to-purple-900/30" />

      {/* Purple orb - slow drift (subtle) */}
      <motion.div
        className="absolute w-[1000px] h-[1000px] rounded-full"
        style={{
          background: "radial-gradient(circle at center, rgba(139,92,246,0.4) 0%, rgba(139,92,246,0.2) 30%, rgba(139,92,246,0.1) 60%, transparent 100%)",
          filter: "blur(80px)",
          willChange: "transform",
          top: "-20%",
          left: "-15%",
        }}
        animate={prefersReducedMotion ? {} : {
          x: ["-10%", "10%", "-10%"],
          y: ["-5%", "15%", "-5%"],
        }}
        transition={{
          duration: 20,
          repeat: Infinity,
          ease: "easeInOut"
        }}
      />

      {/* Blue orb - slower counter-drift (subtle) */}
      <motion.div
        className="absolute w-[900px] h-[900px] rounded-full"
        style={{
          background: "radial-gradient(circle at center, rgba(14,165,233,0.35) 0%, rgba(14,165,233,0.18) 30%, rgba(14,165,233,0.08) 60%, transparent 100%)",
          filter: "blur(80px)",
          willChange: "transform",
          right: "-15%",
          bottom: "-15%",
        }}
        animate={prefersReducedMotion ? {} : {
          x: ["0%", "-20%", "0%"],
          y: ["0%", "-10%", "0%"],
        }}
        transition={{
          duration: 25,
          repeat: Infinity,
          ease: "easeInOut",
          delay: 2
        }}
      />

      {/* Third orb - center drift (subtle) */}
      <motion.div
        className="absolute w-[700px] h-[700px] rounded-full"
        style={{
          background: "radial-gradient(circle at center, rgba(168,85,247,0.3) 0%, rgba(168,85,247,0.15) 40%, rgba(168,85,247,0.06) 70%, transparent 100%)",
          filter: "blur(70px)",
          willChange: "transform",
          top: "30%",
          left: "40%",
        }}
        animate={prefersReducedMotion ? {} : {
          x: ["0%", "15%", "0%"],
          y: ["0%", "-15%", "0%"],
          scale: [1, 1.1, 1],
        }}
        transition={{
          duration: 30,
          repeat: Infinity,
          ease: "easeInOut",
          delay: 5
        }}
      />
    </div>
  );
}
