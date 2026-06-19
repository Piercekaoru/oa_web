import React, { useRef, useState, useEffect } from 'react';
import { ArrowUpRight, Menu, X, Copy, Check, Terminal, Bot, Network, Sparkles } from 'lucide-react';
import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';
import gsap from 'gsap';
import { motion, AnimatePresence } from 'framer-motion';

export function cn(...inputs) {
  return twMerge(clsx(inputs));
}

const API_BASE = import.meta.env.VITE_API_BASE || 'http://localhost:8080';

const navigate = (hash) => { window.location.hash = hash; };

function Header({ isMobileMenuOpen, setIsMobileMenuOpen, currentHash, loggedInUser, onLogout }) {
  return (
    <>
      <header className="flex justify-between items-center w-full relative z-50">
        <a href="#" className="text-white font-semibold text-2xl md:text-[28px] tracking-tight hover:opacity-80 transition-opacity relative z-20">
          OA
        </a>
        
        <div className="hidden md:flex absolute inset-0 items-center justify-center pointer-events-none z-10">
          <nav className="flex items-center bg-white/5 backdrop-blur-md p-1 rounded-full border border-white/10 relative pointer-events-auto">
            {['About', 'Install', 'Pricing', 'Dynamic'].map((item) => {
              const hash = `#${item.toLowerCase()}`;
              const isActive = currentHash === hash || (currentHash === '' && item === 'About');
              return (
                <a
                  key={item}
                  href={hash}
                  className={cn(
                    "relative px-6 py-2 text-[14px] font-medium rounded-full transition-colors duration-300 z-10",
                    isActive 
                      ? "text-black" 
                      : "text-gray-300 hover:text-white hover:bg-white/10"
                  )}
                >
                  {isActive && (
                    <motion.div 
                      layoutId="nav-bubble"
                      className="absolute inset-0 bg-white/90 rounded-full shadow-sm -z-10"
                      transition={{ type: "spring", bounce: 0.15, duration: 0.5 }}
                    />
                  )}
                  <span className="relative z-10">{item}</span>
                </a>
              );
            })}
          </nav>
        </div>
        
        <div className="relative z-20 flex items-center">
          {loggedInUser ? (
            <div className="hidden md:flex items-center gap-6">
              <span className="text-white/60 text-sm">{loggedInUser.email}</span>
              <a href="#dashboard" className="text-white/60 hover:text-white text-[14px] font-medium transition-colors duration-300">
                Dashboard
              </a>
              <button 
                onClick={onLogout}
                className="flex items-center gap-2 bg-white/10 hover:bg-white/20 text-white rounded-full px-5 py-2.5 text-[14px] transition-colors duration-300 border border-white/10"
              >
                Log Out
              </button>
            </div>
          ) : (
            <a href="#auth" className="hidden md:flex items-center gap-2 bg-[#4B66D1] hover:bg-[#3B54B4] text-white rounded-full px-6 py-2.5 text-[14px] transition-colors duration-300 shadow-lg">
              Sign Up / Log In
              <ArrowUpRight className="w-4 h-4 stroke-[1.5]" />
            </a>
          )}
        </div>
        
        <button 
          className="md:hidden text-white bg-white/10 p-2 rounded-full backdrop-blur-md border border-white/5"
          onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
        >
          {isMobileMenuOpen ? <X className="w-6 h-6" /> : <Menu className="w-6 h-6" />}
        </button>
      </header>

      <AnimatePresence>
        {isMobileMenuOpen && (
          <motion.div 
            initial={{ opacity: 0, y: -15, scale: 0.95 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -15, scale: 0.95 }}
            transition={{ duration: 0.25, ease: [0.16, 1, 0.3, 1] }}
            className="absolute top-[80px] left-6 right-6 z-50 bg-[#0a0a0a]/90 backdrop-blur-2xl border border-white/10 rounded-3xl p-6 flex flex-col gap-4 shadow-2xl origin-top"
          >
            {['About', 'Install', 'Pricing', 'Dynamic'].map((item) => (
              <a
                key={item}
                href={`#${item.toLowerCase()}`}
                className="text-white/80 text-xl font-medium hover:text-white transition-colors"
                onClick={() => setIsMobileMenuOpen(false)}
              >
                {item}
              </a>
            ))}
            {loggedInUser && (
              <>
                <a href="#dashboard" onClick={() => setIsMobileMenuOpen(false)} className="text-white/80 text-xl font-medium hover:text-white transition-colors">
                  Dashboard
                </a>
                <button
                  onClick={() => { setIsMobileMenuOpen(false); onLogout(); }}
                  className="mt-4 flex items-center justify-center gap-2 bg-white/10 hover:bg-white/20 text-white rounded-full px-6 py-3.5 text-[16px] font-medium w-full transition-colors border border-white/10"
                >
                  Log Out
                </button>
              </>
            )}
            {!loggedInUser && (
              <a href="#auth" onClick={() => setIsMobileMenuOpen(false)} className="mt-4 flex items-center justify-center gap-2 bg-[#4B66D1] hover:bg-[#3B54B4] text-white rounded-full px-6 py-3.5 text-[16px] font-medium w-full transition-colors shadow-lg">
                Sign Up / Log In
                <ArrowUpRight className="w-4 h-4 stroke-[2]" />
              </a>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </>
  );
}

function HomePage() {
  return (
    <div className="mt-auto relative z-20 flex flex-col w-full">
      <main className="pb-16 md:pb-24 flex flex-col md:flex-row justify-between items-start md:items-end gap-10 lg:gap-20 animate-in fade-in duration-700">
        <div className="flex flex-col space-y-4 md:space-y-6">
          <h1 className="text-white text-[60px] md:text-[86px] tracking-[-0.04em] leading-[0.85] font-medium">
            Open
          </h1>
          <h1 
            className="text-[60px] md:text-[86px] tracking-[-0.04em] leading-[0.85] font-medium text-white/20"
            style={{ WebkitTextStroke: '1.5px rgba(255, 255, 255, 0.9)' }}
          >
            achieve
          </h1>
        </div>
        
        <div className="flex flex-col md:items-end max-w-full md:max-w-[340px] gap-6 lg:gap-8 w-full">
          <p className="text-white/95 text-[15px] md:text-[16px] leading-relaxed font-light text-left">
            Welcome to OpenAchieve. Get started and subscribe to our Agent!
          </p>
          
          <a href="#install" className="group flex justify-between items-center w-full sm:w-[240px] px-6 py-4 rounded-full font-medium shadow-lg bg-white text-black hover:bg-gray-50 transition-all duration-300">
            View Installation
            <ArrowUpRight className="w-5 h-5 stroke-[2] transition-transform duration-300 group-hover:translate-x-1 group-hover:-translate-y-1" />
          </a>
        </div>
      </main>

      {/* Footer */}
      <footer className="w-full pt-6 md:pt-8 pb-4 border-t border-white/10 flex flex-col md:flex-row justify-between items-center gap-6 text-white/40 text-[13px] font-medium animate-in fade-in duration-700">
        <div className="flex flex-wrap items-center justify-center gap-6">
          <a href="#docs" className="hover:text-white transition-colors">Docs</a>
          <a href="#terms" className="hover:text-white transition-colors">Terms of Service</a>
          <a href="#privacy" className="hover:text-white transition-colors">Privacy Policy</a>
          <a href="#data-usage" className="hover:text-white transition-colors">Data Usage</a>
        </div>
        <div className="flex items-center gap-6">
          <a href="https://discord.gg/j5DmQKgME" target="_blank" rel="noopener noreferrer" className="hover:text-white transition-colors flex items-center gap-2">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
              <path d="M20.317 4.3698a19.7913 19.7913 0 00-4.8851-1.5152.0741.0741 0 00-.0785.0371c-.211.3753-.4447.8648-.6083 1.2495-1.8447-.2762-3.68-.2762-5.4868 0-.1636-.3933-.4058-.8742-.6177-1.2495a.077.077 0 00-.0785-.037 19.7363 19.7363 0 00-4.8852 1.515.0699.0699 0 00-.0321.0277C.5334 9.0458-.319 13.5799.0992 18.0578a.0824.0824 0 00.0312.0561c2.0528 1.5076 4.0413 2.4228 5.9929 3.0294a.0777.0777 0 00.0842-.0276c.4616-.6304.8731-1.2952 1.226-1.9942a.076.076 0 00-.0416-.1057c-.6528-.2476-1.2743-.5495-1.8722-.8923a.077.077 0 01-.0076-.1277c.1258-.0943.2517-.1923.3718-.2914a.0743.0743 0 01.0776-.0105c3.9278 1.7933 8.18 1.7933 12.0614 0a.0739.0739 0 01.0785.0095c.1202.099.246.1981.3728.2924a.077.077 0 01-.0066.1276 12.2986 12.2986 0 01-1.873.8914.0766.0766 0 00-.0407.1067c.3604.698.7719 1.3628 1.225 1.9932a.076.076 0 00.0842.0286c1.961-.6067 3.9495-1.5219 6.0023-3.0294a.077.077 0 00.0313-.0552c.5004-5.177-.8382-9.6739-3.5485-13.6604a.061.061 0 00-.0312-.0286zM8.02 15.3312c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9555-2.4189 2.157-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.9555 2.4189-2.1569 2.4189zm7.9748 0c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9554-2.4189 2.1569-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.946 2.4189-2.1568 2.4189z"/>
            </svg>
            Discord
          </a>
          <span>© 2026 OpenAchieve</span>
        </div>
      </footer>
    </div>
  );
}

function InstallPage() {
  const [copied, setCopied] = useState(false);
  const installCmd = "npm install -g --ignore-scripts @openachieve/agent";
  const containerRef = useRef(null);

  useEffect(() => {
    let ctx = gsap.context(() => {
      gsap.fromTo('.install-anim', 
        { y: 80, opacity: 0 }, 
        { 
          y: 0, 
          opacity: 1, 
          duration: 1.2, 
          stagger: 0.15, 
          ease: 'power4.out' 
        }
      );
    }, containerRef);
    return () => ctx.revert();
  }, []);

  const handleCopy = () => {
    navigator.clipboard.writeText(installCmd);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const features = [
    { title: "Multi-Provider LLM", desc: "Unified API supporting OpenAI, Anthropic, Google, AWS Bedrock, and local models seamlessly." },
    { title: "Sub-Agent System", desc: "Delegate complex tasks to specialized agents (scout, planner, worker, reviewer) running in parallel." },
    { title: "Terminal User Interface", desc: "Experience live diff rendering, fuzzy file completion, image pasting, and multi-line editing directly in your terminal." },
    { title: "Highly Extensible", desc: "Write custom skills, inject prompt templates, and build new workflows exclusively using TypeScript." }
  ];

  return (
    <div ref={containerRef} className="mt-16 md:mt-32 relative z-20 flex flex-col pb-32">
      
      {/* Massive Typography Hero */}
      <div className="flex flex-col space-y-4 md:space-y-6">
        <h1 className="install-anim text-white text-[60px] md:text-[100px] lg:text-[140px] tracking-[-0.04em] leading-[0.85] font-medium">
          Install
        </h1>
        <h1 
          className="install-anim text-[60px] md:text-[100px] lg:text-[140px] tracking-[-0.04em] leading-[0.85] font-medium text-white/5"
          style={{ WebkitTextStroke: '2px rgba(255, 255, 255, 0.8)' }}
        >
          Agent
        </h1>
      </div>

      {/* Brutalist Terminal Section */}
      <div className="install-anim mt-24 mb-16 max-w-4xl">
        <p className="text-white/60 text-xl md:text-2xl font-light mb-8 leading-relaxed">
          OpenAchieve Agent is an extensible interactive coding agent CLI. Equip your projects natively by installing it globally via npm.
        </p>
        
        <div className="border-l-2 border-white/20 pl-6 md:pl-10 py-2 group flex items-center justify-between gap-4 md:gap-8">
          <div className="overflow-x-auto flex-1 pb-2 -mb-2 custom-scrollbar">
            <span className="text-white font-mono text-xl md:text-3xl tracking-tight whitespace-nowrap">
              {installCmd}
            </span>
          </div>
          <button 
            onClick={handleCopy} 
            className="text-white/30 hover:text-white transition-colors duration-300 p-2 shrink-0"
          >
            {copied ? <Check className="w-6 h-6 text-green-400" /> : <Copy className="w-6 h-6" />}
          </button>
        </div>
        <div className="mt-6 text-white/40 font-mono text-sm tracking-widest uppercase flex items-center gap-3">
          <div className="w-1 h-1 bg-white/40 rounded-full"></div>
          Node.js 22.19.0 or later
        </div>
      </div>

      {/* Quick Start List */}
      <div className="flex flex-col space-y-16 mt-16 max-w-5xl">
        <div className="install-anim flex flex-col md:flex-row items-start md:items-baseline border-t border-white/10 pt-12 gap-6 md:gap-24 hover:border-white/40 transition-colors duration-500">
          <div className="text-white/30 font-mono text-xl w-12">00</div>
          <div className="flex-1">
            <h3 className="text-white text-3xl md:text-5xl font-medium tracking-tight mb-6">Quick Start</h3>
            <p className="text-white/50 text-xl font-light leading-relaxed mb-6 max-w-2xl">
              Launch the agent inside any project directory to start editing and researching immediately.
            </p>
            <div className="font-mono text-white/80 space-y-2 text-lg">
              <div><span className="text-white/30">$</span> cd /path/to/project</div>
              <div><span className="text-white/30">$</span> oa</div>
            </div>
          </div>
        </div>

        {features.map((f, i) => (
          <div key={i} className="install-anim flex flex-col md:flex-row items-start md:items-baseline border-t border-white/10 pt-12 gap-6 md:gap-24 hover:border-white/40 transition-colors duration-500">
            <div className="text-white/30 font-mono text-xl w-12">0{i + 1}</div>
            <div className="flex-1">
              <h3 className="text-white text-3xl md:text-5xl font-medium tracking-tight mb-6">{f.title}</h3>
              <p className="text-white/50 text-xl md:text-2xl font-light leading-relaxed max-w-2xl">
                {f.desc}
              </p>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function PricingPage({ loggedInUser }) {
  const containerRef = useRef(null);
  const [pending, setPending] = useState(null);
  const [currentPlan, setCurrentPlan] = useState(null);
  const RANK = { free: 0, plus: 1, pro: 2, max: 3 };
  const curRank = RANK[currentPlan] ?? 0;

  useEffect(() => {
    if (!loggedInUser) { setCurrentPlan(null); return; }
    const token = localStorage.getItem('oa_token');
    fetch(`${API_BASE}/api/me`, { headers: { Authorization: `Bearer ${token}` } })
      .then((r) => r.json())
      .then((d) => { if (d.success) setCurrentPlan(d.subscription?.plan || 'free'); })
      .catch(() => {});
  }, [loggedInUser]);

  useEffect(() => {
    let ctx = gsap.context(() => {
      gsap.fromTo('.pricing-anim',
        { y: 80, opacity: 0 },
        {
          y: 0,
          opacity: 1,
          duration: 1.2,
          stagger: 0.1,
          ease: 'power4.out'
        }
      );
    }, containerRef);
    return () => ctx.revert();
  }, []);

  const tiers = [
    { key: "free", name: "Free", price: "$0", desc: "Bring Your Own Key (BYOK) to get started instantly." },
    { key: "plus", name: "Plus", price: "$20", desc: "Includes 2,000 credits for daily tasks with standard models." },
    { key: "pro", name: "Pro", price: "$100", desc: "Includes 10,000 credits for heavy professional workflows." },
    { key: "max", name: "Max", price: "$200", desc: "Includes 20,000 credits and exclusive access to our dynamic fusion models.", glow: true }
  ];

  const pollOrder = async (orderId, token) => {
    for (let i = 0; i < 60; i++) {
      await new Promise((r) => setTimeout(r, 2000));
      try {
        const res = await fetch(`${API_BASE}/api/orders/${orderId}`, {
          headers: { Authorization: `Bearer ${token}` },
        });
        const data = await res.json();
        if (data.success && data.status === 'paid') {
          navigate('#dashboard');
          return;
        }
      } catch { /* keep polling */ }
    }
    setPending(null);
  };

  const handleSubscribe = async (tier) => {
    if (tier.available === false) return;
    if (tier.key === 'free') { navigate('#install'); return; }
    if (!loggedInUser) { navigate('#auth'); return; }

    const token = localStorage.getItem('oa_token');
    setPending(tier.key);
    const payWindow = window.open('', '_blank');
    try {
      const res = await fetch(`${API_BASE}/api/checkout`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token}` },
        body: JSON.stringify({ plan: tier.key }),
      });
      const data = await res.json();
      if (data.success && data.pay_url) {
        if (payWindow) payWindow.location.href = data.pay_url;
        else window.location.href = data.pay_url;
        pollOrder(data.order_id, token);
      } else {
        if (payWindow) payWindow.close();
        setPending(null);
      }
    } catch {
      if (payWindow) payWindow.close();
      setPending(null);
    }
  };

  return (
    <div ref={containerRef} className="mt-16 md:mt-32 relative z-20 flex flex-col pb-32">

      {/* Massive Typography Hero */}
      <div className="flex flex-col space-y-4 md:space-y-6 mb-24">
        <h1 className="pricing-anim text-white text-[60px] md:text-[100px] lg:text-[140px] tracking-[-0.04em] leading-[0.85] font-medium">
          Select
        </h1>
        <h1
          className="pricing-anim text-[60px] md:text-[100px] lg:text-[140px] tracking-[-0.04em] leading-[0.85] font-medium text-white/5"
          style={{ WebkitTextStroke: '2px rgba(255, 255, 255, 0.8)' }}
        >
          Tier
        </h1>
      </div>

      {/* Pricing Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-6 w-full">
        {tiers.map((tier, i) => {
          const isMax = tier.key === 'max';
          const isFree = tier.key === 'free';
          const cardRank = RANK[tier.key];
          const isCurrent = !isFree && currentPlan === tier.key;            // 当前套餐
          const canUpgrade = isMax && currentPlan === 'pro';                // Pro→Max 升级
          const maxGuide = isMax && !isCurrent && !canUpgrade && curRank < RANK.pro; // 引导先订 Pro
          const isDowngrade = !isFree && !isCurrent && !canUpgrade && cardRank < curRank; // 低于当前
          const activeKey = canUpgrade ? 'max_upgrade' : tier.key;
          const isPendingThis = pending === activeKey;
          const disabled = isPendingThis || isCurrent || isDowngrade;
          const handleClick = () => {
            if (isCurrent || isDowngrade) return;
            if (canUpgrade) return handleSubscribe({ key: 'max_upgrade', name: 'Max' });
            if (maxGuide) return handleSubscribe(tiers.find((t) => t.key === 'pro'));
            return handleSubscribe(tier);
          };
          const label = isPendingThis
            ? "Waiting for payment..."
            : isCurrent
              ? "Current Plan"
              : isDowngrade
                ? "Included"
                : isFree
                  ? "Start Free"
                  : canUpgrade
                    ? "Upgrade · +$100"
                    : maxGuide
                      ? "Subscribe Pro first"
                      : "Subscribe";
          return (
          <div key={i} className={cn(
            "pricing-anim relative flex flex-col justify-between p-8 md:p-10 rounded-[2rem] border transition-all duration-500 group",
            tier.glow
              ? "bg-white/10 border-blue-400/30 hover:border-blue-400 shadow-[0_0_40px_rgba(59,130,246,0.1)] hover:shadow-[0_0_60px_rgba(59,130,246,0.3)]"
              : "bg-white/[0.02] border-white/10 hover:bg-white/[0.05] hover:border-white/30"
          )}>
            <div>
              <div className="flex justify-between items-center mb-12">
                <h3 className="text-white/60 text-lg tracking-widest uppercase font-medium">{tier.name}</h3>
                {tier.glow && <div className="w-2 h-2 rounded-full bg-blue-400 shadow-[0_0_10px_rgba(96,165,250,0.8)] animate-pulse" />}
              </div>
              <div className="mb-8">
                <span className="text-white text-6xl md:text-7xl font-medium tracking-tight">{tier.price}</span>
                <span className="text-white/40 text-lg ml-2">/mo</span>
              </div>
              <p className={cn("text-white/60 text-lg font-light leading-relaxed", isMax ? "mb-4" : "mb-12")}>
                {tier.desc}
              </p>
              {isMax && (
                <p className="text-blue-300/70 text-sm font-light leading-relaxed mb-12">
                  {isCurrent
                    ? "You're on the Max plan."
                    : canUpgrade
                      ? "You're on Pro — add $100 to upgrade to Max."
                      : "Subscribe to Pro first, then add $100 to upgrade to Max."}
                </p>
              )}
            </div>

            <button
              onClick={handleClick}
              disabled={disabled}
              className={cn(
              "w-full py-4 rounded-full font-medium transition-all duration-300 flex justify-center items-center gap-2 group-hover:gap-4",
              tier.glow
                ? "bg-blue-500 text-white hover:bg-blue-400 shadow-lg"
                : "bg-white/10 text-white hover:bg-white hover:text-black",
              disabled && "opacity-60 cursor-not-allowed"
            )}>
              {label}
              {!disabled && <ArrowUpRight className="w-5 h-5 stroke-[2]" />}
            </button>
          </div>
          );
        })}
      </div>
    </div>
  );
}

function DynamicPage() {
  const containerRef = useRef(null);

  useEffect(() => {
    let ctx = gsap.context(() => {
      gsap.fromTo('.dynamic-anim', 
        { y: 80, opacity: 0 }, 
        { 
          y: 0, 
          opacity: 1, 
          duration: 1.2, 
          stagger: 0.15, 
          ease: 'power4.out' 
        }
      );
    }, containerRef);
    return () => ctx.revert();
  }, []);

  const faqs = [
    { 
      q: "What is Dynamic?", 
      a: "Dynamic is a cutting-edge multi-model fusion engine. It intelligently routes and fuses outputs across models, delivering top-tier reasoning capabilities at exactly half the price of leading models (like Claude Fable 5)." 
    },
    { 
      q: "Can I specify the models to fuse?", 
      a: "Model selection is fully automated. Custom model specification is not currently available, ensuring optimal performance and latency out-of-the-box without manual configuration." 
    },
    {
      q: "Which subscriptions include this?",
      a: "Dynamic will be available to MAX and PRO tier subscribers. It's launching soon — stay tuned."
    }
  ];

  return (
    <div ref={containerRef} className="mt-16 md:mt-32 relative z-20 flex flex-col pb-32">

      {/* Coming Soon Text */}
      <div className="dynamic-anim self-start mb-6">
        <p className="text-transparent bg-clip-text bg-gradient-to-r from-white/30 via-white/50 to-white/30 text-[13px] font-light tracking-[0.5em] uppercase">
          — Coming Soon
        </p>
      </div>

      {/* Massive Typography Hero */}
      <div className="flex flex-col space-y-4 md:space-y-6 mb-24">
        <h1 className="dynamic-anim text-white text-[60px] md:text-[100px] lg:text-[140px] tracking-[-0.04em] leading-[0.85] font-medium">
          Dynamic
        </h1>
        <h1 
          className="dynamic-anim text-[60px] md:text-[100px] lg:text-[140px] tracking-[-0.04em] leading-[0.85] font-medium text-white/5"
          style={{ WebkitTextStroke: '2px rgba(255, 255, 255, 0.8)' }}
        >
          Fusion
        </h1>
      </div>

      {/* Hero Description */}
      <div className="dynamic-anim mb-24 max-w-4xl border-l-2 border-white/20 pl-6 md:pl-10 py-2">
        <p className="text-white/80 text-2xl md:text-4xl font-light leading-relaxed tracking-tight">
          Top-tier results. <span className="text-blue-400 font-medium">Half the price.</span>
        </p>
      </div>

      {/* Brutalist Q&A List */}
      <div className="flex flex-col space-y-16 max-w-5xl">
        {faqs.map((faq, i) => (
          <div key={i} className="dynamic-anim flex flex-col md:flex-row items-start border-t border-white/10 pt-12 gap-6 md:gap-24 hover:border-white/40 transition-colors duration-500 group">
            <div className="text-white/20 group-hover:text-blue-400/50 transition-colors duration-500 font-mono text-3xl md:text-5xl tracking-tighter w-16">
              Q{i + 1}
            </div>
            <div className="flex-1">
              <h3 className="text-white text-3xl md:text-4xl font-medium tracking-tight mb-6">{faq.q}</h3>
              <p className="text-white/50 text-xl md:text-2xl font-light leading-relaxed max-w-3xl">
                {faq.a}
              </p>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function AuthPage({ onLoginSuccess }) {
  const [mode, setMode] = useState('login'); // 'login' | 'register' | 'verify'
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [code, setCode] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const containerRef = useRef(null);

  const isLogin = mode === 'login';
  const isRegister = mode === 'register';
  const isVerify = mode === 'verify';
  const isForgot = mode === 'forgot';
  const isReset = mode === 'reset';

  useEffect(() => {
    let ctx = gsap.context(() => {
      gsap.fromTo('.auth-anim',
        { y: 60, opacity: 0 },
        {
          y: 0,
          opacity: 1,
          duration: 1,
          stagger: 0.1,
          ease: 'power4.out'
        }
      );
    }, containerRef);
    return () => ctx.revert();
  }, [mode]);

  const switchMode = (m) => (e) => {
    e.preventDefault();
    setMode(m);
    setError('');
    setSuccess('');
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    setLoading(true);
    setError('');
    setSuccess('');

    try {
      if (isLogin) {
        const res = await fetch(`${API_BASE}/api/login`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ email, password }),
        });
        const data = await res.json();

        if (data.success && data.token) {
          localStorage.setItem('oa_token', data.token);
          localStorage.setItem('oa_user', JSON.stringify(data.user));
          setSuccess('Login successful! Redirecting...');
          setTimeout(() => {
            if (onLoginSuccess) onLoginSuccess(data.user);
            window.location.hash = '';
          }, 800);
        } else if (res.status === 403 && /verif/i.test(data.message || '')) {
          setMode('verify');
          setSuccess('Please enter the verification code sent to your email.');
        } else {
          setError(data.message || 'Login failed.');
        }
      } else if (isRegister) {
        const res = await fetch(`${API_BASE}/api/register`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ name, email, password }),
        });
        const data = await res.json();

        if (data.success) {
          setMode('verify');
          setSuccess('We sent a 6-digit code to your email. Enter it below to finish.');
        } else {
          setError(data.message || 'Registration failed.');
        }
      } else if (isVerify) {
        const res = await fetch(`${API_BASE}/api/verify`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ email, code: code.trim() }),
        });
        const data = await res.json();

        if (data.success) {
          setMode('login');
          setCode('');
          setSuccess('Email verified! Please log in.');
        } else {
          setError(data.message || 'Verification failed.');
        }
      } else if (isForgot) {
        const res = await fetch(`${API_BASE}/api/forgot-password`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ email }),
        });
        const data = await res.json();

        if (data.success) {
          setMode('reset');
          setSuccess('If an account exists, a reset code was sent to your email.');
        } else {
          setError(data.message || 'Request failed.');
        }
      } else if (isReset) {
        const res = await fetch(`${API_BASE}/api/reset-password`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ email, code: code.trim(), password }),
        });
        const data = await res.json();

        if (data.success) {
          setMode('login');
          setCode('');
          setPassword('');
          setSuccess('Password reset! Please log in.');
        } else {
          setError(data.message || 'Reset failed.');
        }
      }
    } catch (err) {
      setError('Unable to connect to the server. Is the backend running?');
    } finally {
      setLoading(false);
    }
  };

  const heroWord = isRegister ? 'Up' : 'In';
  const subtitle = isLogin
    ? 'Welcome back. Enter your credentials to access your agents and dynamic models.'
    : isRegister
      ? 'Create an account to unlock our cutting-edge multi-provider LLM workflows.'
      : isVerify
        ? 'Check your email and enter the 6-digit code to finish signing up.'
        : isForgot
          ? "Forgot your password? Enter your email and we'll send you a reset code."
          : 'Enter the code from your email and choose a new password.';
  const submitLabel = isLogin
    ? 'Log In'
    : isRegister
      ? 'Create Account'
      : isVerify
        ? 'Verify Email'
        : isForgot
          ? 'Send Reset Code'
          : 'Reset Password';

  return (
    <div ref={containerRef} className="mt-8 md:mt-20 relative z-20 flex flex-col md:flex-row items-center justify-center pb-32 gap-16 lg:gap-24 xl:gap-40 w-full max-w-6xl mx-auto flex-1 h-full">

      {/* Left side: Massive Typography Hero */}
      <div className="flex flex-col space-y-4 md:space-y-6 w-full md:w-1/2 max-w-lg">
        <h1 className="auth-anim text-white text-[60px] md:text-[100px] lg:text-[140px] tracking-[-0.04em] leading-[0.85] font-medium">
          Sign
        </h1>
        <h1
          className="auth-anim text-[60px] md:text-[100px] lg:text-[140px] tracking-[-0.04em] leading-[0.85] font-medium text-white/5"
          style={{ WebkitTextStroke: '2px rgba(255, 255, 255, 0.8)' }}
        >
          {heroWord}
        </h1>
        <p className="auth-anim text-white/50 text-lg md:text-xl font-light mt-8 max-w-md">
          {subtitle}
        </p>
      </div>

      {/* Right side: Brutalist Form */}
      <div className="auth-anim w-full md:w-1/2 max-w-md bg-white/10 border border-white/20 p-8 md:p-12 rounded-[2rem] backdrop-blur-2xl shadow-2xl relative overflow-hidden">
        <div className="absolute top-0 right-0 w-64 h-64 bg-blue-500/20 blur-[100px] rounded-full pointer-events-none" />

        {/* Status Messages */}
        {error && (
          <div className="relative z-10 mb-6 text-red-400 text-sm bg-red-500/10 border border-red-500/20 rounded-xl px-4 py-3">
            {error}
          </div>
        )}
        {success && (
          <div className="relative z-10 mb-8 flex items-center justify-center gap-3 px-6 py-4 rounded-2xl border border-white/10 bg-white/[0.03] backdrop-blur-md shadow-2xl animate-in fade-in zoom-in duration-300">
            <div className="flex items-center justify-center w-6 h-6 rounded-full bg-blue-500/20 border border-blue-500/30">
              <Check className="w-3.5 h-3.5 text-blue-400" />
            </div>
            <span className="text-white/90 text-[14px] font-medium tracking-wide">
              {success}
            </span>
          </div>
        )}

        <form className="flex flex-col gap-10 relative z-10" onSubmit={handleSubmit}>
          {isRegister && (
            <div className="flex flex-col group">
              <label className="text-white/60 text-xs font-mono tracking-widest uppercase mb-2 group-focus-within:text-blue-400 transition-colors">Full Name</label>
              <input
                type="text"
                placeholder="John Doe"
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="bg-transparent border-b border-white/30 pb-2 text-white text-lg font-light focus:outline-none focus:border-blue-400 transition-colors placeholder:text-white/20"
              />
            </div>
          )}

          {isVerify || isReset ? (
            <>
              <div className="flex flex-col">
                <label className="text-white/60 text-xs font-mono tracking-widest uppercase mb-2">Email Address</label>
                <div className="text-white text-lg font-light border-b border-white/15 pb-2 truncate">{email}</div>
              </div>
              <div className="flex flex-col group">
                <label className="text-white/60 text-xs font-mono tracking-widest uppercase mb-2 group-focus-within:text-blue-400 transition-colors">{isReset ? 'Reset Code' : 'Verification Code'}</label>
                <input
                  type="text"
                  inputMode="numeric"
                  maxLength={6}
                  placeholder="000000"
                  value={code}
                  onChange={(e) => setCode(e.target.value.replace(/\D/g, ''))}
                  className="bg-transparent border-b border-white/30 pb-2 text-white text-2xl font-mono tracking-[0.4em] focus:outline-none focus:border-blue-400 transition-colors placeholder:text-white/20"
                />
              </div>
              {isReset && (
                <div className="flex flex-col group">
                  <label className="text-white/60 text-xs font-mono tracking-widest uppercase mb-2 group-focus-within:text-blue-400 transition-colors">New Password</label>
                  <input
                    type="password"
                    placeholder="••••••••"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    className="bg-transparent border-b border-white/30 pb-2 text-white text-lg tracking-widest focus:outline-none focus:border-blue-400 transition-colors placeholder:text-white/20"
                  />
                </div>
              )}
            </>
          ) : (
            <>
              <div className="flex flex-col group">
                <label className="text-white/60 text-xs font-mono tracking-widest uppercase mb-2 group-focus-within:text-blue-400 transition-colors">Email Address</label>
                <input
                  type="email"
                  placeholder="hello@openachieve.com"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  className="bg-transparent border-b border-white/30 pb-2 text-white text-lg font-light focus:outline-none focus:border-blue-400 transition-colors placeholder:text-white/20"
                />
              </div>

              {!isForgot && (
                <div className="flex flex-col group">
                  <div className="flex justify-between items-center mb-2">
                    <label className="text-white/60 text-xs font-mono tracking-widest uppercase group-focus-within:text-blue-400 transition-colors">Password</label>
                    {isLogin && <a href="#" onClick={switchMode('forgot')} className="text-white/40 text-xs hover:text-white transition-colors">Forgot?</a>}
                  </div>
                  <input
                    type="password"
                    placeholder="••••••••"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    className="bg-transparent border-b border-white/30 pb-2 text-white text-lg tracking-widest focus:outline-none focus:border-blue-400 transition-colors placeholder:text-white/20"
                  />
                </div>
              )}
            </>
          )}

          <button
            type="submit"
            disabled={loading}
            className={cn(
              "mt-4 w-full bg-blue-600 hover:bg-blue-500 text-white font-medium py-4 rounded-full shadow-[0_0_20px_rgba(37,99,235,0.2)] hover:shadow-[0_0_30px_rgba(37,99,235,0.4)] transition-all duration-300 flex justify-center items-center gap-2",
              loading && "opacity-60 cursor-not-allowed"
            )}
          >
            {loading ? (
              <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
            ) : (
              <>
                {submitLabel}
                <ArrowUpRight className="w-5 h-5 stroke-[2]" />
              </>
            )}
          </button>
        </form>

        <div className="mt-8 text-center relative z-10">
          {(isVerify || isForgot || isReset) ? (
            <button onClick={switchMode('login')} className="text-white/40 text-sm hover:text-white transition-colors">
              Back to log in
            </button>
          ) : (
            <button onClick={switchMode(isLogin ? 'register' : 'login')} className="text-white/40 text-sm hover:text-white transition-colors">
              {isLogin ? "Don't have an account? Sign up" : "Already have an account? Log in"}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

function DashboardPage({ user, onLogout }) {
  const containerRef = useRef(null);
  const [sub, setSub] = useState(null);
  const [loading, setLoading] = useState(() => !!localStorage.getItem('oa_token'));

  useEffect(() => {
    let ctx = gsap.context(() => {
      gsap.fromTo('.dash-anim',
        { y: 40, opacity: 0 },
        {
          y: 0,
          opacity: 1,
          duration: 1,
          stagger: 0.1,
          ease: 'power4.out'
        }
      );
    }, containerRef);
    return () => ctx.revert();
  }, [loading]);

  useEffect(() => {
    const token = localStorage.getItem('oa_token');
    if (!token) return;
    fetch(`${API_BASE}/api/me`, { headers: { Authorization: `Bearer ${token}` } })
      .then((res) => res.json())
      .then((data) => { if (data.success) setSub(data.subscription); })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  const plan = sub?.plan ? sub.plan.charAt(0).toUpperCase() + sub.plan.slice(1) : 'Free';
  const credits = sub?.credits ?? 0;
  const expiry = sub?.expires || '—';

  return (
    <div ref={containerRef} className="mt-16 md:mt-24 relative z-20 flex flex-col pb-32 max-w-6xl w-full mx-auto">
      <div className="flex flex-col space-y-4 md:space-y-6 mb-16">
        <h1 className="dash-anim text-white text-[50px] md:text-[80px] tracking-[-0.04em] leading-[0.85] font-medium">
          Dashboard
        </h1>
        <p className="dash-anim text-white/50 text-xl font-light">
          Welcome back, {user?.name || 'User'}
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Account Details */}
        <div className="dash-anim bg-white/5 border border-white/10 rounded-[2rem] p-8 md:p-12 backdrop-blur-xl">
          <h3 className="text-white/40 text-xs font-mono tracking-widest uppercase mb-10">Account Details</h3>

          <div className="space-y-8">
            <div>
              <div className="text-white/40 text-sm mb-2">Account ID</div>
              <div className="text-white font-mono text-lg truncate selection:bg-blue-500/30">
                {user?.id || '—'}
              </div>
            </div>
            <div>
              <div className="text-white/40 text-sm mb-2">Email Address</div>
              <div className="text-white text-xl">{user?.email || '—'}</div>
            </div>
          </div>

          <button
            onClick={onLogout}
            className="mt-10 flex items-center justify-center gap-2 bg-white/10 hover:bg-white/20 text-white rounded-full px-6 py-3 text-sm font-medium transition-colors border border-white/10"
          >
            Log Out
          </button>
        </div>

        {/* Subscription & Credits */}
        <div className="dash-anim bg-gradient-to-br from-blue-500/10 to-purple-500/10 border border-blue-500/20 rounded-[2rem] p-8 md:p-12 backdrop-blur-xl relative overflow-hidden group">
          <div className="absolute top-0 right-0 w-[400px] h-[400px] bg-blue-500/20 blur-[100px] rounded-full pointer-events-none transition-transform duration-700 group-hover:scale-110" />

          <h3 className="text-blue-400/80 text-xs font-mono tracking-widest uppercase mb-10 relative z-10">Subscription</h3>

          <div className="space-y-10 relative z-10">
            <div className="flex flex-wrap gap-8 justify-between items-end border-b border-white/10 pb-8">
              <div>
                <div className="text-white/40 text-sm mb-2">Current Plan</div>
                <div className="text-white text-4xl md:text-5xl font-medium tracking-tight">{loading ? '…' : plan}</div>
              </div>
              <div className="text-left md:text-right">
                <div className="text-white/40 text-sm mb-2">Expires</div>
                <div className="text-white text-xl font-mono">{loading ? '…' : expiry}</div>
              </div>
            </div>

            <div className="flex flex-wrap gap-6 justify-between items-end">
              <div>
                <div className="text-white/40 text-sm mb-2">Credit Balance</div>
                <div className="text-white text-5xl md:text-6xl font-medium tracking-tight">
                  {loading ? '…' : credits.toLocaleString()}
                </div>
              </div>
              <a href="#pricing" className="bg-blue-500 hover:bg-blue-400 text-white rounded-full px-8 py-3 transition-colors text-sm font-medium shadow-lg hover:shadow-blue-500/25">
                Top Up
              </a>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function DevicePage({ loggedInUser, initialCode }) {
  const containerRef = useRef(null);
  const [code, setCode] = useState(initialCode || '');
  const [status, setStatus] = useState('idle'); // idle | loading | success | error
  const [message, setMessage] = useState('');

  useEffect(() => {
    let ctx = gsap.context(() => {
      gsap.fromTo('.device-anim',
        { y: 60, opacity: 0 },
        { y: 0, opacity: 1, duration: 1, stagger: 0.1, ease: 'power4.out' }
      );
    }, containerRef);
    return () => ctx.revert();
  }, []);

  const handleAuthorize = async (e) => {
    e.preventDefault();
    if (!loggedInUser) { navigate('#auth'); return; }
    const token = localStorage.getItem('oa_token');
    setStatus('loading');
    setMessage('');
    try {
      const res = await fetch(`${API_BASE}/api/auth/device/approve`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token}` },
        body: JSON.stringify({ user_code: code.trim() }),
      });
      const data = await res.json();
      if (data.success) {
        setStatus('success');
        setMessage('Device authorized. Return to your terminal — the CLI will finish logging in automatically.');
      } else {
        setStatus('error');
        setMessage(data.message || 'Authorization failed.');
      }
    } catch {
      setStatus('error');
      setMessage('Unable to connect to the server. Is the backend running?');
    }
  };

  return (
    <div ref={containerRef} className="mt-8 md:mt-20 relative z-20 flex flex-col md:flex-row items-center justify-center pb-32 gap-16 lg:gap-24 xl:gap-40 w-full max-w-6xl mx-auto flex-1 h-full">

      {/* Left side: Massive Typography Hero */}
      <div className="flex flex-col space-y-4 md:space-y-6 w-full md:w-1/2 max-w-lg">
        <h1 className="device-anim text-white text-[60px] md:text-[100px] lg:text-[140px] tracking-[-0.04em] leading-[0.85] font-medium">
          Connect
        </h1>
        <h1
          className="device-anim text-[60px] md:text-[100px] lg:text-[140px] tracking-[-0.04em] leading-[0.85] font-medium text-white/5"
          style={{ WebkitTextStroke: '2px rgba(255, 255, 255, 0.8)' }}
        >
          Device
        </h1>
        <p className="device-anim text-white/50 text-lg md:text-xl font-light mt-8 max-w-md">
          Confirm the code shown in your terminal to link the OpenAchieve CLI to your account.
        </p>
      </div>

      {/* Right side: Brutalist Form */}
      <div className="device-anim w-full md:w-1/2 max-w-md bg-white/10 border border-white/20 p-8 md:p-12 rounded-[2rem] backdrop-blur-2xl shadow-2xl relative overflow-hidden">
        <div className="absolute top-0 right-0 w-64 h-64 bg-blue-500/20 blur-[100px] rounded-full pointer-events-none" />

        <div className="relative z-10 flex items-center gap-3 mb-8 text-white/60">
          <Terminal className="w-5 h-5" />
          <span className="text-xs font-mono tracking-widest uppercase">Device Authorization</span>
        </div>

        {!loggedInUser && (
          <div className="relative z-10 mb-6 text-blue-300 text-sm bg-blue-500/10 border border-blue-500/20 rounded-xl px-4 py-3">
            Please <a href="#auth" className="underline hover:text-white">log in</a> first, then return to this page.
          </div>
        )}
        {status === 'error' && (
          <div className="relative z-10 mb-6 text-red-400 text-sm bg-red-500/10 border border-red-500/20 rounded-xl px-4 py-3">
            {message}
          </div>
        )}
        {status === 'success' && (
          <div className="relative z-10 mb-6 text-green-400 text-sm bg-green-500/10 border border-green-500/20 rounded-xl px-4 py-3">
            {message}
          </div>
        )}

        <form className="flex flex-col gap-10 relative z-10" onSubmit={handleAuthorize}>
          <div className="flex flex-col group">
            <label className="text-white/60 text-xs font-mono tracking-widest uppercase mb-2 group-focus-within:text-blue-400 transition-colors">Device Code</label>
            <input
              type="text"
              placeholder="XXXX-XXXX"
              value={code}
              onChange={(e) => setCode(e.target.value.toUpperCase())}
              className="bg-transparent border-b border-white/30 pb-2 text-white text-2xl font-mono tracking-[0.3em] focus:outline-none focus:border-blue-400 transition-colors placeholder:text-white/20"
            />
          </div>

          <button
            type="submit"
            disabled={status === 'loading' || status === 'success'}
            className={cn(
              "w-full bg-blue-600 hover:bg-blue-500 text-white font-medium py-4 rounded-full shadow-[0_0_20px_rgba(37,99,235,0.2)] hover:shadow-[0_0_30px_rgba(37,99,235,0.4)] transition-all duration-300 flex justify-center items-center gap-2",
              (status === 'loading' || status === 'success') && "opacity-60 cursor-not-allowed"
            )}
          >
            {status === 'loading' ? (
              <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
            ) : status === 'success' ? (
              <>Authorized <Check className="w-5 h-5 stroke-[2]" /></>
            ) : (
              <>Authorize Device <ArrowUpRight className="w-5 h-5 stroke-[2]" /></>
            )}
          </button>
        </form>
      </div>
    </div>
  );
}

function GenericPage({ title, children }) {
  return (
    <div className="mt-16 md:mt-24 relative z-20 flex flex-col max-w-4xl mx-auto w-full pb-32 animate-in fade-in duration-700">
      <h1 className="text-white text-4xl md:text-5xl font-medium tracking-tight mb-8 md:mb-12">
        {title}
      </h1>
      <div className="text-white/70 text-[16px] leading-relaxed space-y-6">
        {children}
      </div>
    </div>
  );
}

function H2({ children }) {
  return <h2 className="text-white text-2xl font-medium tracking-tight pt-4">{children}</h2>;
}

function Code({ children }) {
  return <code className="text-blue-300 bg-white/5 px-1.5 py-0.5 rounded text-[14px]">{children}</code>;
}

function Cmd({ children }) {
  return <pre className="bg-white/5 border border-white/10 rounded-xl p-4 overflow-x-auto text-[13px] text-white/80 leading-relaxed">{children}</pre>;
}

function DocsPage() {
  return (
    <GenericPage title="Documentation">
      <p>OpenAchieve is a coding agent you run in your terminal. Subscribe once, sign in, and spend your credits to run powerful models on your own machine — with built-in safety, parallel subagents, autonomous goal mode, and full provider flexibility.</p>

      <H2>Getting Started</H2>
      <p>Install the CLI globally, then run it inside any project directory:</p>
      <Cmd>{`npm install -g --ignore-scripts @openachieve/agent
oa`}</Cmd>
      <p>Sign in with <Code>/login</Code> and choose <strong className="text-white">OpenAchieve</strong>. A browser window opens for device authorization — enter the displayed code to link your account. Check your plan and credit balance from your <a className="text-blue-300 hover:text-blue-200 underline" href="#dashboard">Dashboard</a>, and switch models anytime with <Code>/model</Code>.</p>

      <H2>Subscription &amp; Credits</H2>
      <p>Paid plans grant a monthly allotment of credits; every model request draws from that balance. Credits are valid for the 30-day billing period and do not roll over.</p>
      <ul className="list-disc pl-5 space-y-1">
        <li><strong className="text-white">Plus / Pro / Max</strong> — increasing monthly credits. See <a className="text-blue-300 hover:text-blue-200 underline" href="#pricing">Pricing</a>.</li>
        <li><strong className="text-white">Max</strong> unlocks our Dynamic fusion models. Already on Pro? Upgrade to Max for an additional $100 from the pricing page.</li>
      </ul>

      <H2>Bring Your Own Provider</H2>
      <p>Prefer your own API key or a third-party relay? Point OpenAchieve at any OpenAI- or Anthropic-compatible endpoint via <Code>~/.openachieve/agent/config.toml</Code>:</p>
      <Cmd>{`[providers.my-relay]
baseUrl = "https://relay.example.com/v1"
api = "openai-completions"   # or "anthropic-messages"
apiKey = "sk-..."
models = ["gpt-4o", "o3"]

[settings]
defaultModel = "my-relay/gpt-4o"`}</Cmd>
      <p>Register as many providers and models as you like, then switch between them with <Code>/model</Code>.</p>

      <H2>Permissions &amp; Safety</H2>
      <p>Every action that touches your filesystem or runs a command passes through a permission layer. Choose how strict it is:</p>
      <ul className="list-disc pl-5 space-y-1">
        <li><Code>ask</Code> — confirm sensitive actions interactively (default).</li>
        <li><Code>allow</Code> — auto-approve, while still blocking credential files.</li>
        <li><Code>bypass</Code> — skip prompts entirely (trusted environments only).</li>
      </ul>
      <p>Set it with <Code>oa --permission-mode &lt;mode&gt;</Code> or the <Code>permission</Code> block in <Code>settings.json</Code>. Secrets such as <Code>.env</Code>, <Code>~/.ssh/*</Code>, and <Code>*.pem</Code> are protected by default.</p>

      <H2>Sandboxing &amp; Containers</H2>
      <p>For extra isolation, run OpenAchieve inside a sandbox or container so tools execute against a contained filesystem, process space, and network. You can containerize the entire <Code>oa</Code> process with Docker, or use a sandbox to scope what the agent can reach. Permissions and sandboxing act as two independent safety layers.</p>

      <H2>Subagents</H2>
      <p>Delegate work to specialized agents that run in parallel and report back. List what's available with <Code>/agents</Code>, then:</p>
      <ul className="list-disc pl-5 space-y-1">
        <li><Code>/run &lt;agent&gt; [task]</Code> — run a single subagent.</li>
        <li><Code>/parallel</Code> / <Code>/chain</Code> — fan out concurrently or pipe one into the next.</li>
        <li><Code>/view-agent</Code> — watch a running subagent's conversation live.</li>
      </ul>
      <p>Built-in roles include scout, planner, worker, and reviewer; runs can be backgrounded or forked from the current session.</p>

      <H2>Goal Mode</H2>
      <p>Hand OpenAchieve an objective and let it iterate autonomously until done. Start with <Code>/goal &lt;intent&gt;</Code>, or launch via <Code>oa --goal "..."</Code>. An independent judge model verifies your completion criteria so the agent can't declare success prematurely. Steer a run with <Code>/goal status</Code>, <Code>pause</Code>, <Code>resume</Code>, or <Code>cancel</Code>.</p>

      <H2>Plan Mode</H2>
      <p>Toggle <Code>/plan</Code> (or <Code>Ctrl+Alt+P</Code>) to enter a read-only mode: the agent explores your code and proposes a numbered plan, and only executes after you approve it — ideal for high-stakes changes.</p>

      <H2>MCP (Model Context Protocol)</H2>
      <p>Connect external tools through MCP servers. Configure them in <Code>~/.openachieve/agent/mcp.json</Code>:</p>
      <Cmd>{`{
  "mcpServers": {
    "my-tool": { "command": "npx", "args": ["-y", "my-mcp-server"] },
    "remote":  { "url": "https://mcp.example.com" }
  }
}`}</Cmd>
      <p>Inspect connections with <Code>/mcp</Code> and list exposed tools with <Code>/mcp tools</Code>.</p>

      <H2>Extensions</H2>
      <p>Extend OpenAchieve with TypeScript extensions to add commands, providers, or behaviors — load one with <Code>oa -e &lt;path&gt;</Code> or drop it into your extensions directory.</p>
    </GenericPage>
  );
}

function TermsPage() {
  return (
    <GenericPage title="Terms of Service">
      <p className="text-white/40 text-sm">Last updated: June 20, 2026</p>
      <p>These Terms of Service ("Terms") govern your access to and use of OpenAchieve, including the <Code>oa</Code> command-line agent, the OpenAchieve website, and related subscription services (collectively, the "Service"). By creating an account or using the Service, you agree to these Terms.</p>

      <H2>1. Eligibility &amp; Accounts</H2>
      <p>You must be able to form a binding contract to use the Service. You are responsible for the information you provide, for keeping your account credentials secure, and for all activity under your account.</p>

      <H2>2. The Service</H2>
      <p>OpenAchieve provides a coding agent that, with your subscription, routes your model requests to third-party model providers to generate responses. Features, models, and limits may change over time as we improve the Service.</p>

      <H2>3. Subscriptions &amp; Credits</H2>
      <ul className="list-disc pl-5 space-y-1">
        <li>Paid plans grant an allotment of <strong className="text-white">credits</strong>, measured in credits rather than currency. Model usage consumes credits.</li>
        <li>Credits are valid for the 30-day billing period and <strong className="text-white">do not roll over</strong>; any unused balance expires at the end of the period.</li>
        <li>Except where required by law, payments and credits are <strong className="text-white">non-refundable</strong>.</li>
        <li>Payments are handled by a third-party payment provider; we do not store your full payment details.</li>
        <li>We may change pricing or plan contents prospectively; changes do not affect a period you have already paid for.</li>
      </ul>

      <H2>4. Acceptable Use</H2>
      <p>You agree not to use the Service to violate any law, infringe others' rights, generate harmful or abusive content, resell or redistribute access, circumvent credit metering or usage limits, or attempt to disrupt or reverse-engineer the Service.</p>

      <H2>5. Bring-Your-Own Keys &amp; Relays</H2>
      <p>If you configure your own API keys or third-party relays, your use of those providers is governed by their terms, and you are responsible for any associated costs and compliance.</p>

      <H2>6. Intellectual Property</H2>
      <p>You retain rights to the inputs you provide and the outputs you generate, to the extent permitted by applicable law and any third-party provider terms. OpenAchieve and its software, branding, and content remain our property.</p>

      <H2>7. Disclaimers</H2>
      <p>The Service is provided "as is" without warranties of any kind. AI-generated output may be inaccurate or incomplete — review and verify it before relying on it, especially for code that modifies your systems.</p>

      <H2>8. Limitation of Liability</H2>
      <p>To the maximum extent permitted by law, OpenAchieve is not liable for indirect, incidental, or consequential damages, or for any loss of data, profits, or business arising from your use of the Service.</p>

      <H2>9. Termination</H2>
      <p>You may stop using the Service at any time. We may suspend or terminate access for violations of these Terms or to protect the Service and its users.</p>

      <H2>10. Changes to These Terms</H2>
      <p>We may update these Terms from time to time. Material changes will be reflected by the "Last updated" date above; continued use constitutes acceptance.</p>

      <H2>11. Governing Law</H2>
      <p>These Terms are governed by the laws of [Governing jurisdiction — to be specified], without regard to conflict-of-law principles.</p>

      <H2>12. Contact</H2>
      <p>Questions about these Terms? Contact us at <a className="text-blue-300 hover:text-blue-200 underline" href="mailto:kayano04@proton.me">kayano04@proton.me</a>.</p>
    </GenericPage>
  );
}

function PrivacyPage() {
  return (
    <GenericPage title="Privacy Policy">
      <p className="text-white/40 text-sm">Last updated: June 20, 2026</p>
      <p>This Privacy Policy explains what information OpenAchieve collects, how we use it, and the choices you have. It applies to the OpenAchieve website, the <Code>oa</Code> agent, and related services.</p>

      <H2>Information We Collect</H2>
      <ul className="list-disc pl-5 space-y-1">
        <li><strong className="text-white">Account data</strong> — your email address and name.</li>
        <li><strong className="text-white">Authentication</strong> — login and session tokens used to keep you signed in.</li>
        <li><strong className="text-white">Subscription &amp; usage</strong> — your plan, credit balance, and usage metadata (measured in credits).</li>
        <li><strong className="text-white">Payment status</strong> — confirmation of payments from our payment provider. We do not store your card details.</li>
        <li><strong className="text-white">Content you submit</strong> — the prompts, code, and conversation data you send while using the agent.</li>
        <li><strong className="text-white">Diagnostics</strong> — minimal, operational telemetry to keep the Service reliable.</li>
      </ul>

      <H2>How We Use It</H2>
      <p>We use this information to operate and provide the Service, process payments, maintain your account and credit balance, prevent abuse, provide support, and improve reliability.</p>

      <H2>Sharing</H2>
      <p>We share data only as needed to run the Service: with third-party model providers (to fulfill your model requests), our payment processor, our email-delivery provider, and our hosting infrastructure. We do not sell your personal data.</p>

      <H2>Data Retention</H2>
      <p>We retain account data while your account is active, and conversation or session data to power features like history and your dashboard. You can request deletion of your data at any time.</p>

      <H2>Security</H2>
      <p>We protect data in transit with encryption and apply access controls. Your agent credentials are stored locally on your own device.</p>

      <H2>Your Rights</H2>
      <p>You may request to access, correct, export, or delete your personal data by contacting us, and we will respond consistent with applicable law.</p>

      <H2>International Transfers</H2>
      <p>Your data may be processed in countries other than where you live. Where required, we use appropriate safeguards for such transfers.</p>

      <H2>Children</H2>
      <p>The Service is not directed to children and is not intended for use by anyone under the age required by their local law to consent to data processing.</p>

      <H2>Changes</H2>
      <p>We may update this Policy; material changes will be reflected by the "Last updated" date above.</p>

      <H2>Contact</H2>
      <p>For privacy questions or requests, contact <a className="text-blue-300 hover:text-blue-200 underline" href="mailto:kayano04@proton.me">kayano04@proton.me</a>.</p>
    </GenericPage>
  );
}

function DataUsagePage() {
  return (
    <GenericPage title="Data Usage">
      <p className="text-white/40 text-sm">Last updated: June 20, 2026</p>
      <p>This page describes specifically how the prompts, code, and other content you send to OpenAchieve are handled.</p>

      <H2>How Your Prompts &amp; Code Are Used</H2>
      <p>To generate responses, the content you submit is transmitted to third-party model providers that process it on our behalf to fulfill your request. Processing is performed solely to produce the output you asked for.</p>

      <H2>Training</H2>
      <p>We do <strong className="text-white">not</strong> use your private code or prompts to train our own foundation models. Third-party providers process your requests under their own policies. If you bring your own keys or relays, that traffic is governed by the provider you choose.</p>

      <H2>Retention</H2>
      <p>Your conversations and sessions are stored to power features such as history, context compaction, and your dashboard. You can delete this data at any time.</p>

      <H2>Telemetry</H2>
      <p>Any diagnostics we collect are minimal and operational (for example, error signals and usage counts in credits). Telemetry does <strong className="text-white">not</strong> include the content of your prompts.</p>

      <H2>Your Control</H2>
      <p>You can review or delete your stored data, and you control provider configuration and permissions from within the agent. For data requests, contact <a className="text-blue-300 hover:text-blue-200 underline" href="mailto:kayano04@proton.me">kayano04@proton.me</a>.</p>
    </GenericPage>
  );
}

export default function App() {
  const [currentHash, setCurrentHash] = useState(window.location.hash);
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);
  const [loggedInUser, setLoggedInUser] = useState(() => {
    try {
      const stored = localStorage.getItem('oa_user');
      return stored ? JSON.parse(stored) : null;
    } catch { return null; }
  });
  const glowRef = useRef(null);
  const containerRef = useRef(null);

  useEffect(() => {
    const handleHashChange = () => setCurrentHash(window.location.hash);
    window.addEventListener('hashchange', handleHashChange);
    return () => window.removeEventListener('hashchange', handleHashChange);
  }, []);

  const handleLogout = () => {
    localStorage.removeItem('oa_token');
    localStorage.removeItem('oa_user');
    setLoggedInUser(null);
    navigate('#');
  };

  const handleMouseMove = (e) => {
    if (!glowRef.current || !containerRef.current) return;
    const rect = containerRef.current.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    
    glowRef.current.style.setProperty('--mx', `${x}px`);
    glowRef.current.style.setProperty('--my', `${y}px`);
  };

  const handleMouseEnter = () => {
    if (glowRef.current) {
      glowRef.current.style.setProperty('--r', '220px');
    }
  };

  const handleMouseLeave = () => {
    if (glowRef.current) {
      glowRef.current.style.setProperty('--r', '0px');
    }
  };

  const isInstallPage = currentHash === '#install';
  const isPricingPage = currentHash === '#pricing';
  const isDynamicPage = currentHash === '#dynamic';
  const isAuthPage = currentHash === '#auth';
  const isDashboardPage = currentHash === '#dashboard';
  const isDocsPage = currentHash === '#docs';
  const isTermsPage = currentHash === '#terms';
  const isPrivacyPage = currentHash === '#privacy';
  const isDataUsagePage = currentHash === '#data-usage';
  const isDevicePage = currentHash.startsWith('#device');
  const deviceCodeParam = (currentHash.match(/[?&]code=([^&]+)/) || [])[1];
  const initialDeviceCode = deviceCodeParam ? decodeURIComponent(deviceCodeParam) : '';
  const isOverlayPage = isInstallPage || isPricingPage || isDynamicPage || isAuthPage || isDashboardPage || isDevicePage || isDocsPage || isTermsPage || isPrivacyPage || isDataUsagePage;

  return (
    <div 
      ref={containerRef}
      className={cn(
        "min-h-screen w-full relative flex flex-col overflow-x-hidden bg-black antialiased",
        isOverlayPage ? "overflow-y-auto" : "overflow-hidden"
      )}
      onMouseMove={handleMouseMove}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      {/* Global Background Video (always plays) */}
      <div className="fixed inset-0 z-0 pointer-events-none">
        <video 
          autoPlay 
          loop 
          muted 
          playsInline 
          className="object-cover w-full h-full"
          src="/hero-bg.mp4"
        />
        {/* Dynamic overlay depending on page */}
        <div className={cn(
          "absolute inset-0 transition-all duration-1000",
          isOverlayPage 
            ? "bg-black/80 backdrop-blur-xl" 
            : "bg-gradient-to-t from-black/80 via-black/20 to-transparent"
        )} />
      </div>

      {/* Global Interactive Organic Glow Effect (z-10) */}
      <div 
        ref={glowRef}
        className="pointer-events-none fixed inset-0 z-10 transition-opacity duration-1000"
        style={{
          '--mx': '50%',
          '--my': '50%',
          background: 'radial-gradient(var(--r, 0px) circle at var(--mx) var(--my), rgba(80,150,255,0.35), transparent)',
          transition: '--r 0.8s cubic-bezier(0.25, 1, 0.5, 1)',
          opacity: isOverlayPage ? 0.3 : 1
        }}
      />

      {/* Content Wrapper */}
      <div className={cn(
        "mx-auto max-w-[1920px] w-full flex flex-col flex-1 relative z-20 p-6 md:p-8 lg:p-12",
        !isOverlayPage && "justify-between"
      )}>
        <Header 
          isMobileMenuOpen={isMobileMenuOpen} 
          setIsMobileMenuOpen={setIsMobileMenuOpen} 
          currentHash={currentHash}
          loggedInUser={loggedInUser}
          onLogout={handleLogout}
        />
        
        {isInstallPage && <InstallPage />}
        {isPricingPage && <PricingPage loggedInUser={loggedInUser} />}
        {isDynamicPage && <DynamicPage />}
        {isAuthPage && <AuthPage onLoginSuccess={(user) => setLoggedInUser(user)} />}
        {isDashboardPage && <DashboardPage user={loggedInUser} onLogout={handleLogout} />}
        {isDevicePage && <DevicePage loggedInUser={loggedInUser} initialCode={initialDeviceCode} />}
        {isDocsPage && <DocsPage />}
        {isTermsPage && <TermsPage />}
        {isPrivacyPage && <PrivacyPage />}
        {isDataUsagePage && <DataUsagePage />}
        {!isOverlayPage && <HomePage />}
      </div>
    </div>
  );
}
