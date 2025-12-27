import { NavLink } from 'react-router-dom';
import { Home, History, Settings, Sparkles, Bookmark } from 'lucide-react';
import { motion } from 'framer-motion';

export function Sidebar() {
  const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  const navItems = [
    { path: '/', icon: Home, label: 'Today' },
    { path: '/history', icon: History, label: 'History' },
    { path: '/bookmarks', icon: Bookmark, label: 'Bookmarks' },
    { path: '/settings', icon: Settings, label: 'Settings' },
  ];

  return (
    <aside className="w-64 h-screen backdrop-blur-xl bg-white/5 dark:bg-white/5 border-r border-white/10 flex flex-col">
      <div className="p-6 border-b border-white/10">
        <div className="flex items-center gap-3">
          <motion.div
            className="w-10 h-10 bg-gradient-to-br from-primary-500 to-primary-700 rounded-xl flex items-center justify-center"
            animate={prefersReducedMotion ? {} : {
              boxShadow: [
                "0 0 20px rgba(14, 165, 233, 0.3)",
                "0 0 30px rgba(14, 165, 233, 0.5)",
                "0 0 20px rgba(14, 165, 233, 0.3)",
              ]
            }}
            transition={{ duration: 3, repeat: Infinity }}
          >
            <Sparkles className="w-6 h-6 text-white" />
          </motion.div>
          <div>
            <h1 className="text-xl font-bold text-gray-900 dark:text-white">Claudius</h1>
            <p className="text-xs text-gray-500 dark:text-gray-400">AI Research Agent</p>
          </div>
        </div>
      </div>

      <nav className="flex-1 p-4">
        <ul className="space-y-2">
          {navItems.map((item) => {
            const Icon = item.icon;
            return (
              <li key={item.path}>
                <motion.div whileHover={prefersReducedMotion ? {} : { scale: 1.02 }} transition={{ duration: 0.2 }}>
                  <NavLink
                    to={item.path}
                    className={({ isActive }) =>
                      `flex items-center gap-3 px-4 py-3 rounded-lg transition-all duration-200 ${
                        isActive
                          ? 'glass-nav-active text-primary-400'
                          : 'glass-nav text-gray-400'
                      }`
                    }
                  >
                    <Icon className="w-5 h-5" />
                    <span className="font-medium">{item.label}</span>
                  </NavLink>
                </motion.div>
              </li>
            );
          })}
        </ul>
      </nav>

      <div className="p-4 border-t border-white/10">
        <div className="text-xs text-gray-400 dark:text-gray-500 text-center">
          v0.5.1
        </div>
      </div>
    </aside>
  );
}
