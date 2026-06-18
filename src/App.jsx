import React, { useRef, useState, useEffect } from 'react';
import { ArrowUpRight, Menu, X, Copy, Check, Terminal, Bot, Network, Sparkles } from 'lucide-react';
import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';
import gsap from 'gsap';
import { motion } from 'framer-motion';

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

      {isMobileMenuOpen && (
        <div className="absolute top-[80px] left-6 right-6 z-50 bg-black/80 backdrop-blur-xl border border-white/10 rounded-2xl p-6 flex flex-col gap-4 animate-in fade-in zoom-in duration-200 shadow-2xl">
          {['About', 'Install', 'Pricing', 'Dynamic'].map((item) => (
            <a
              key={item}
              href={`#${item.toLowerCase()}`}
              className="text-white text-lg font-medium hover:text-white/70"
              onClick={() => setIsMobileMenuOpen(false)}
            >
              {item}
            </a>
          ))}
          {loggedInUser && (
            <a href="#dashboard" onClick={() => setIsMobileMenuOpen(false)} className="text-white text-lg font-medium hover:text-white/70">
              Dashboard
            </a>
          )}
          {!loggedInUser && (
            <a href="#auth" onClick={() => setIsMobileMenuOpen(false)} className="mt-4 flex items-center justify-center gap-2 bg-[#4B66D1] text-white rounded-full px-6 py-3 text-[16px] font-medium w-full">
              Sign Up / Log In
              <ArrowUpRight className="w-4 h-4 stroke-[2]" />
            </a>
          )}
        </div>
      )}
    </>
  );
}

function HomePage() {
  return (
    <main className="mt-auto relative z-20 pb-4 flex flex-col md:flex-row justify-between items-start md:items-end gap-10 lg:gap-20 animate-in fade-in duration-700">
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
    { key: "max", name: "Max", price: "$200", desc: "Includes 20,000 credits and exclusive access to our dynamic fusion models.", glow: true, available: false }
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
        {tiers.map((tier, i) => (
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
              <p className="text-white/60 text-lg font-light leading-relaxed mb-12">
                {tier.desc}
              </p>
            </div>

            <button
              onClick={() => handleSubscribe(tier)}
              disabled={pending === tier.key || tier.available === false}
              className={cn(
              "w-full py-4 rounded-full font-medium transition-all duration-300 flex justify-center items-center gap-2 group-hover:gap-4",
              tier.glow
                ? "bg-blue-500 text-white hover:bg-blue-400 shadow-lg"
                : "bg-white/10 text-white hover:bg-white hover:text-black",
              (pending === tier.key || tier.available === false) && "opacity-60 cursor-not-allowed"
            )}>
              {tier.available === false
                ? "Coming Soon"
                : pending === tier.key
                  ? "Waiting for payment..."
                  : tier.price === "$0" ? "Start Free" : "Subscribe"}
              {tier.available !== false && pending !== tier.key && <ArrowUpRight className="w-5 h-5 stroke-[2]" />}
            </button>
          </div>
        ))}
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

      {/* Coming Soon badge */}
      <div className="dynamic-anim inline-flex items-center gap-2 self-start mb-6 px-4 py-2 rounded-full border border-blue-400/30 bg-blue-500/10">
        <span className="w-2 h-2 rounded-full bg-blue-400 animate-pulse" />
        <span className="text-blue-300 text-sm font-medium tracking-widest uppercase">Coming Soon</span>
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
      } else {
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
      }
    } catch (err) {
      setError('Unable to connect to the server. Is the backend running?');
    } finally {
      setLoading(false);
    }
  };

  const heroWord = isLogin ? 'In' : 'Up';
  const subtitle = isLogin
    ? 'Welcome back. Enter your credentials to access your agents and dynamic models.'
    : isRegister
      ? 'Create an account to unlock our cutting-edge multi-provider LLM workflows.'
      : 'Check your email and enter the 6-digit code to finish signing up.';
  const submitLabel = isLogin ? 'Log In' : isRegister ? 'Create Account' : 'Verify Email';

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
          <div className="relative z-10 mb-6 text-green-400 text-sm bg-green-500/10 border border-green-500/20 rounded-xl px-4 py-3">
            {success}
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

          {isVerify ? (
            <>
              <div className="flex flex-col">
                <label className="text-white/60 text-xs font-mono tracking-widest uppercase mb-2">Email Address</label>
                <div className="text-white text-lg font-light border-b border-white/15 pb-2 truncate">{email}</div>
              </div>
              <div className="flex flex-col group">
                <label className="text-white/60 text-xs font-mono tracking-widest uppercase mb-2 group-focus-within:text-blue-400 transition-colors">Verification Code</label>
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

              <div className="flex flex-col group">
                <div className="flex justify-between items-center mb-2">
                  <label className="text-white/60 text-xs font-mono tracking-widest uppercase group-focus-within:text-blue-400 transition-colors">Password</label>
                  {isLogin && <a href="#" className="text-white/40 text-xs hover:text-white transition-colors">Forgot?</a>}
                </div>
                <input
                  type="password"
                  placeholder="••••••••"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="bg-transparent border-b border-white/30 pb-2 text-white text-lg tracking-widest focus:outline-none focus:border-blue-400 transition-colors placeholder:text-white/20"
                />
              </div>
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
          {isVerify ? (
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

function DashboardPage({ user }) {
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
  const isDevicePage = currentHash.startsWith('#device');
  const deviceCodeParam = (currentHash.match(/[?&]code=([^&]+)/) || [])[1];
  const initialDeviceCode = deviceCodeParam ? decodeURIComponent(deviceCodeParam) : '';
  const isOverlayPage = isInstallPage || isPricingPage || isDynamicPage || isAuthPage || isDashboardPage || isDevicePage;

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
        {isDashboardPage && <DashboardPage user={loggedInUser} />}
        {isDevicePage && <DevicePage loggedInUser={loggedInUser} initialCode={initialDeviceCode} />}
        {!isOverlayPage && <HomePage />}
      </div>
    </div>
  );
}
