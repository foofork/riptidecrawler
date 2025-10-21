#!/usr/bin/env python3
"""
CLI Quantitative Analysis Script
Analyzes all CLI modules for LOC, dependencies, complexity, and categorization
"""

import os
import re
import json
from pathlib import Path
from collections import defaultdict
from dataclasses import dataclass, asdict
from typing import List, Dict, Set

@dataclass
class ModuleMetrics:
    name: str
    loc: int
    functions: int
    structs: int
    enums: int
    impls: int
    imports: int
    singletons: int
    riptide_deps: List[str]
    external_deps: List[str]
    category: str
    complexity_score: float

class CLIAnalyzer:
    def __init__(self, base_path: str):
        self.base_path = Path(base_path)
        self.modules: List[ModuleMetrics] = []

    def analyze_file(self, filepath: Path) -> ModuleMetrics:
        """Analyze a single Rust file"""
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()

        name = filepath.stem

        # Count metrics
        loc = len([l for l in content.split('\n') if l.strip() and not l.strip().startswith('//')])
        functions = len(re.findall(r'^\s*(pub\s+)?(async\s+)?fn\s+', content, re.MULTILINE))
        structs = len(re.findall(r'^\s*(pub\s+)?struct\s+', content, re.MULTILINE))
        enums = len(re.findall(r'^\s*(pub\s+)?enum\s+', content, re.MULTILINE))
        impls = len(re.findall(r'^impl\s+', content, re.MULTILINE))
        imports = len(re.findall(r'^use\s+', content, re.MULTILINE))

        # Singleton patterns
        singleton_patterns = [
            r'\bstatic\b',
            r'\bOnceCell\b',
            r'\bGLOBAL\b',
            r'Arc<Mutex',
            r'lazy_static',
            r'Arc<RwLock'
        ]
        singletons = sum(len(re.findall(pattern, content)) for pattern in singleton_patterns)

        # Riptide dependencies
        riptide_imports = re.findall(r'^use riptide_(\w+)', content, re.MULTILINE)
        riptide_deps = list(set(riptide_imports))

        # External dependencies
        external_imports = re.findall(r'^use (\w+)::', content, re.MULTILINE)
        external_deps = list(set([
            dep for dep in external_imports
            if dep not in ['std', 'super', 'crate', 'riptide']
        ]))

        # Calculate complexity score
        complexity = (
            (functions * 1.0) +
            (structs * 0.5) +
            (enums * 0.5) +
            (impls * 0.8) +
            (len(riptide_deps) * 2.0) +
            (len(external_deps) * 1.5) +
            (singletons * 3.0)  # Singletons add significant complexity
        )

        # Categorize
        category = self.categorize_module(name, content)

        return ModuleMetrics(
            name=name,
            loc=loc,
            functions=functions,
            structs=structs,
            enums=enums,
            impls=impls,
            imports=imports,
            singletons=singletons,
            riptide_deps=riptide_deps,
            external_deps=external_deps,
            category=category,
            complexity_score=complexity
        )

    def categorize_module(self, name: str, content: str) -> str:
        """Categorize module by purpose"""
        infrastructure_keywords = ['pool', 'cache', 'timeout', 'manager', 'monitor', 'health']
        business_keywords = ['extract', 'render', 'crawl', 'pdf', 'tables', 'search']
        config_keywords = ['domain', 'session', 'schema', 'stealth', 'validate', 'system_check']
        management_keywords = ['metrics', 'progress', 'job']
        wasm_keywords = ['wasm', 'aot']

        name_lower = name.lower()

        if any(kw in name_lower for kw in wasm_keywords):
            return 'WASM/Runtime'
        elif any(kw in name_lower for kw in infrastructure_keywords):
            return 'Infrastructure'
        elif any(kw in name_lower for kw in business_keywords):
            return 'Business Logic'
        elif any(kw in name_lower for kw in config_keywords):
            return 'Configuration'
        elif any(kw in name_lower for kw in management_keywords):
            return 'Management'
        elif name == 'mod':
            return 'Core/Routing'
        else:
            return 'Utility'

    def analyze_all(self):
        """Analyze all CLI modules"""
        cli_commands = self.base_path / 'crates' / 'riptide-cli' / 'src' / 'commands'

        for rs_file in sorted(cli_commands.glob('*.rs')):
            metrics = self.analyze_file(rs_file)
            self.modules.append(metrics)

    def generate_dependency_matrix(self) -> Dict[str, Set[str]]:
        """Generate dependency relationships"""
        matrix = defaultdict(set)

        for module in self.modules:
            for dep in module.riptide_deps:
                matrix[module.name].add(dep)

        return dict(matrix)

    def calculate_roi(self, module: ModuleMetrics) -> Dict[str, any]:
        """Calculate extraction ROI for a module"""
        # Business logic modules have highest extraction value
        business_multiplier = 3.0 if module.category == 'Business Logic' else 1.0

        # Modules with many dependencies are risky to extract
        dependency_penalty = len(module.riptide_deps) * 0.3

        # Singletons make extraction harder
        singleton_penalty = module.singletons * 2.0

        # Size penalty (larger = harder to extract)
        size_penalty = (module.loc / 100) * 0.5

        # Calculate extraction difficulty (0-10 scale)
        difficulty = min(10, dependency_penalty + singleton_penalty + size_penalty)

        # Calculate value (0-10 scale)
        value = min(10, (module.loc / 100) * business_multiplier)

        # ROI = Value / Difficulty
        roi = value / max(1, difficulty)

        return {
            'value': round(value, 2),
            'difficulty': round(difficulty, 2),
            'roi': round(roi, 2),
            'category': module.category
        }

    def generate_report(self) -> str:
        """Generate comprehensive markdown report"""
        total_loc = sum(m.loc for m in self.modules)
        total_functions = sum(m.functions for m in self.modules)
        total_singletons = sum(m.singletons for m in self.modules)

        # Category breakdown
        category_stats = defaultdict(lambda: {'count': 0, 'loc': 0, 'complexity': 0})
        for m in self.modules:
            category_stats[m.category]['count'] += 1
            category_stats[m.category]['loc'] += m.loc
            category_stats[m.category]['complexity'] += m.complexity_score

        # Dependency analysis
        dep_matrix = self.generate_dependency_matrix()

        # ROI analysis
        roi_rankings = sorted(
            [(m.name, self.calculate_roi(m)) for m in self.modules],
            key=lambda x: x[1]['roi'],
            reverse=True
        )

        report = f"""# CLI Module Quantitative Analysis

**Generated:** {Path.cwd()}
**Total Modules Analyzed:** {len(self.modules)}

## Executive Summary

| Metric | Value |
|--------|-------|
| **Total Lines of Code** | {total_loc:,} |
| **Total Functions** | {total_functions} |
| **Total Singleton Patterns** | {total_singletons} |
| **Avg LOC per Module** | {total_loc // len(self.modules)} |
| **Avg Complexity Score** | {sum(m.complexity_score for m in self.modules) / len(self.modules):.1f} |

## Module Breakdown by LOC

| # | Module | LOC | Funcs | Structs | Enums | Impls | Deps | Singletons | Category | Complexity |
|---|--------|-----|-------|---------|-------|-------|------|------------|----------|------------|
"""

        for i, m in enumerate(sorted(self.modules, key=lambda x: x.loc, reverse=True), 1):
            report += f"| {i:2d} | `{m.name}` | {m.loc:,} | {m.functions} | {m.structs} | {m.enums} | {m.impls} | {len(m.riptide_deps)} | {m.singletons} | {m.category} | {m.complexity_score:.1f} |\n"

        report += f"""

## Category Analysis

| Category | Modules | Total LOC | Avg Complexity | % of Codebase |
|----------|---------|-----------|----------------|---------------|
"""

        for cat, stats in sorted(category_stats.items(), key=lambda x: x[1]['loc'], reverse=True):
            pct = (stats['loc'] / total_loc) * 100
            avg_complexity = stats['complexity'] / stats['count']
            report += f"| {cat} | {stats['count']} | {stats['loc']:,} | {avg_complexity:.1f} | {pct:.1f}% |\n"

        report += """

## Dependency Matrix

### Riptide Crate Dependencies

| Module | Dependencies |
|--------|--------------|
"""

        for module, deps in sorted(dep_matrix.items()):
            if deps:
                report += f"| `{module}` | {', '.join(sorted(deps))} |\n"

        report += """

## Singleton/Global State Analysis

**⚠️ Modules with Global State (Extraction Risk)**

| Module | Singleton Count | Type | Risk Level |
|--------|----------------|------|------------|
"""

        singleton_modules = [m for m in self.modules if m.singletons > 0]
        for m in sorted(singleton_modules, key=lambda x: x.singletons, reverse=True):
            risk = 'HIGH' if m.singletons >= 5 else 'MEDIUM' if m.singletons >= 3 else 'LOW'
            report += f"| `{m.name}` | {m.singletons} | {m.category} | **{risk}** |\n"

        report += f"""

**Total modules with singletons:** {len(singleton_modules)}/{len(self.modules)} ({len(singleton_modules)/len(self.modules)*100:.1f}%)

## Extraction ROI Analysis

**Top 15 Extraction Candidates by ROI**

| Rank | Module | Value | Difficulty | ROI | Category | LOC |
|------|--------|-------|------------|-----|----------|-----|
"""

        for i, (name, roi_data) in enumerate(roi_rankings[:15], 1):
            module = next(m for m in self.modules if m.name == name)
            report += f"| {i:2d} | `{name}` | {roi_data['value']:.1f} | {roi_data['difficulty']:.1f} | **{roi_data['roi']:.2f}** | {roi_data['category']} | {module.loc} |\n"

        report += """

### ROI Scoring Methodology

- **Value**: Business impact × (LOC/100)
  - Business Logic modules: 3× multiplier
  - Other modules: 1× multiplier
- **Difficulty**: Dependencies + Singletons + Size
  - Riptide deps: 0.3 per dep
  - Singletons: 2.0 per singleton
  - Size: 0.5 per 100 LOC
- **ROI**: Value / Difficulty (higher is better)

## Complexity Distribution

"""

        low_complexity = [m for m in self.modules if m.complexity_score < 10]
        med_complexity = [m for m in self.modules if 10 <= m.complexity_score < 30]
        high_complexity = [m for m in self.modules if m.complexity_score >= 30]

        report += f"""| Complexity Level | Count | Modules |
|-----------------|-------|---------|
| Low (<10) | {len(low_complexity)} | {', '.join('`'+m.name+'`' for m in low_complexity[:5])}{'...' if len(low_complexity) > 5 else ''} |
| Medium (10-30) | {len(med_complexity)} | {', '.join('`'+m.name+'`' for m in med_complexity[:5])}{'...' if len(med_complexity) > 5 else ''} |
| High (≥30) | {len(high_complexity)} | {', '.join('`'+m.name+'`' for m in high_complexity[:5])}{'...' if len(high_complexity) > 5 else ''} |

## External Dependencies Analysis

**Most Common External Crates:**

"""

        # Count external dep usage
        ext_dep_count = defaultdict(int)
        for m in self.modules:
            for dep in m.external_deps:
                ext_dep_count[dep] += 1

        for dep, count in sorted(ext_dep_count.items(), key=lambda x: x[1], reverse=True)[:10]:
            report += f"- `{dep}`: Used by {count} modules\n"

        report += """

## Extraction Priority Recommendations

### Phase 1: High-ROI Business Logic (Minimal Dependencies)
"""

        phase1 = [name for name, roi in roi_rankings[:5]]
        for name in phase1:
            m = next(mod for mod in self.modules if mod.name == name)
            report += f"- ✅ `{m.name}` ({m.loc} LOC, {len(m.riptide_deps)} deps, {m.singletons} singletons)\n"

        report += """

### Phase 2: Infrastructure with Moderate Dependencies
"""

        infra_modules = [m for m in self.modules if m.category == 'Infrastructure' and m.singletons <= 3]
        for m in sorted(infra_modules, key=lambda x: x.complexity_score)[:5]:
            report += f"- ⚠️ `{m.name}` ({m.loc} LOC, {len(m.riptide_deps)} deps, {m.singletons} singletons)\n"

        report += """

### Phase 3: High-Singleton Modules (Risky, Defer)
"""

        risky = [m for m in singleton_modules if m.singletons >= 5]
        for m in sorted(risky, key=lambda x: x.singletons, reverse=True)[:5]:
            report += f"- 🚨 `{m.name}` ({m.singletons} singletons, {m.loc} LOC) - **HIGH RISK**\n"

        report += """

## Coupling Analysis

### Highly Coupled Modules (>3 Riptide Dependencies)
"""

        coupled = [m for m in self.modules if len(m.riptide_deps) > 3]
        for m in sorted(coupled, key=lambda x: len(x.riptide_deps), reverse=True):
            report += f"- `{m.name}`: {len(m.riptide_deps)} dependencies → {', '.join(m.riptide_deps)}\n"

        report += """

## Recommendations

1. **Start with low-hanging fruit**: Extract `tables`, `health`, `search` first (low complexity, high ROI)
2. **Avoid singleton modules initially**: Defer `adaptive_timeout`, `browser_pool_manager`, `wasm_cache`
3. **Infrastructure last**: Keep `domain`, `schema`, `session` until Phase 2+
4. **Test infrastructure first**: Ensure testing framework can handle extracted modules
5. **Incremental approach**: Extract 1-2 modules per sprint, validate thoroughly

## Data Summary

- **Total analyzed modules:** {len(self.modules)}
- **Business logic modules:** {category_stats['Business Logic']['count']}
- **Infrastructure modules:** {category_stats['Infrastructure']['count']}
- **Modules with singletons:** {len(singleton_modules)}
- **High-complexity modules:** {len(high_complexity)}
- **Extraction candidates (ROI > 1.0):** {len([r for r in roi_rankings if r[1]['roi'] > 1.0])}

---

**Analysis complete.** Use this data to prioritize extraction efforts and minimize risk.
"""

        return report

def main():
    analyzer = CLIAnalyzer('/workspaces/eventmesh')
    analyzer.analyze_all()

    report = analyzer.generate_report()

    output_path = Path('/workspaces/eventmesh/docs/hive/CLI-QUANTITATIVE-ANALYSIS.md')
    output_path.parent.mkdir(parents=True, exist_ok=True)

    with open(output_path, 'w') as f:
        f.write(report)

    print(f"✅ Analysis complete: {output_path}")
    print(f"📊 Analyzed {len(analyzer.modules)} modules")
    print(f"📈 Total LOC: {sum(m.loc for m in analyzer.modules):,}")

if __name__ == '__main__':
    main()
