<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Inertia\Inertia;

class NewsController extends Controller
{
    private const PER_PAGE = 20;

    public function index()
    {
        $latest = $this->articles(1)['articles'];

        $sentiment = rescue(fn () => DB::table('sentiment_history')
            ->select('asset', 'sentiment_score', 'confidence', 'news_count')
            ->whereRaw("time >= now() - interval '24 hours'")
            ->orderByDesc('time')->get()->groupBy('asset')
            ->map(fn ($g) => $g->first())->values()->toArray(), []);

        $events = rescue(fn () => DB::table('market_events')
            ->select('event_type', 'severity', 'asset', 'headline', 'detected_at')
            ->orderByDesc('detected_at')->limit(15)->get()->toArray(), []);

        $econ = rescue(fn () => DB::table('economic_events')
            ->select('name', 'country', 'importance', 'scheduled_at')
            ->whereRaw('scheduled_at >= now()')
            ->orderBy('scheduled_at')->limit(10)->get()->toArray(), []);

        $stats = rescue(fn () => (array) DB::table('news_articles as a')
            ->leftJoin('news_ai_analysis as an', 'an.article_id', '=', 'a.id')
            ->selectRaw("count(*) total, count(distinct a.source) sources,
                count(*) filter (where an.bias is not null) analyzed,
                count(*) filter (where an.bias='bullish') bullish,
                count(*) filter (where an.bias='bearish') bearish")
            ->whereRaw("a.created_at >= now() - interval '48 hours'")->first(), []);

        return Inertia::render('News', compact('latest', 'sentiment', 'events', 'econ', 'stats'));
    }

    /** JSON feed for infinite scroll. Newest first, optional asset/search filter. */
    public function feed(Request $request)
    {
        $page = max(1, (int) $request->query('page', 1));
        return response()->json($this->articles($page, $request->query('asset'), $request->query('q')));
    }

    private function articles(int $page, ?string $asset = null, ?string $q = null): array
    {
        $query = DB::table('news_articles as a')
            ->leftJoin('news_ai_analysis as an', 'an.article_id', '=', 'a.id')
            ->select('a.id', 'a.title', 'a.source', 'a.url', 'a.summary', 'a.categories',
                'a.assets', 'a.published_at', 'a.created_at',
                'an.bias', 'an.score', 'an.confidence')
            ->orderByRaw('coalesce(a.published_at, a.created_at) desc');

        if ($asset && $asset !== 'all') {
            $query->whereRaw('? = ANY(a.assets)', [strtoupper($asset)]);
        }
        if ($q) {
            $query->where('a.title', 'ilike', '%'.$q.'%');
        }

        $rows = rescue(fn () => $query->offset(($page - 1) * self::PER_PAGE)
            ->limit(self::PER_PAGE + 1)->get(), collect());
        $hasMore = $rows->count() > self::PER_PAGE;

        return [
            'articles' => $rows->take(self::PER_PAGE)->map(fn ($a) => [
                'id' => $a->id,
                'title' => $a->title,
                'source' => $a->source,
                'url' => $a->url,
                'summary' => $a->summary,
                'assets' => $this->pgArray($a->assets),
                'bias' => $a->bias,
                'score' => $a->score !== null ? (float) $a->score : null,
                'confidence' => $a->confidence !== null ? (float) $a->confidence : null,
                'published_at' => $a->published_at ?: $a->created_at,
            ])->values(),
            'next' => $hasMore ? $page + 1 : null,
        ];
    }

    private function pgArray($val): array
    {
        if (is_array($val)) {
            return array_values($val);
        }
        $s = trim((string) $val, '{}');
        return $s === '' ? [] : array_map(fn ($x) => trim($x, '"'), str_getcsv($s));
    }
}
