import { ReactNode, useRef } from 'react';
import { motion, useMotionValue, useSpring } from 'framer-motion';

interface MagneticButtonProps {
  children: ReactNode;
  className?: string;
  onClick?: () => void;
  disabled?: boolean;
  variant?: 'primary' | 'secondary';
}

/**
 * Button with magnetic effect - follows cursor with spring physics.
 * Respects prefers-reduced-motion for accessibility.
 */
export function MagneticButton({
  children,
  className = '',
  onClick,
  disabled,
  variant = 'primary'
}: MagneticButtonProps) {
  const ref = useRef<HTMLButtonElement>(null);
  const x = useMotionValue(0);
  const y = useMotionValue(0);

  const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  const springConfig = prefersReducedMotion
    ? { damping: 100, stiffness: 500 } // Instant
    : { damping: 20, stiffness: 300 };

  const springX = useSpring(x, springConfig);
  const springY = useSpring(y, springConfig);

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!ref.current || disabled || prefersReducedMotion) return;

    const rect = ref.current.getBoundingClientRect();
    const centerX = rect.left + rect.width / 2;
    const centerY = rect.top + rect.height / 2;

    // 30% magnetic pull, max 12px
    const distanceX = Math.max(-12, Math.min(12, (e.clientX - centerX) * 0.3));
    const distanceY = Math.max(-12, Math.min(12, (e.clientY - centerY) * 0.3));

    x.set(distanceX);
    y.set(distanceY);
  };

  const handleMouseLeave = () => {
    x.set(0);
    y.set(0);
  };

  const baseClasses = variant === 'primary' ? 'btn-primary' : 'btn-secondary';

  return (
    <motion.button
      ref={ref}
      className={`btn ${baseClasses} relative overflow-hidden flex items-center justify-center ${className}`}
      onMouseMove={handleMouseMove}
      onMouseLeave={handleMouseLeave}
      onClick={onClick}
      disabled={disabled}
      style={{ x: springX, y: springY }}
      whileHover={!disabled && !prefersReducedMotion ? { scale: 1.05 } : {}}
      whileTap={!disabled && !prefersReducedMotion ? { scale: 0.95 } : {}}
    >
      {/* Glow effect on hover */}
      <motion.div
        className="absolute inset-0 opacity-0 pointer-events-none"
        whileHover={{ opacity: 1 }}
        transition={{ duration: 0.2 }}
        style={{
          background: variant === 'primary'
            ? 'radial-gradient(circle at center, rgba(139,92,246,0.3), transparent)'
            : 'radial-gradient(circle at center, rgba(255,255,255,0.1), transparent)'
        }}
      />
      <span className="relative z-10 flex items-center gap-2">{children}</span>
    </motion.button>
  );
}
