# embed-files (ef)

`ef`は、複数のソースコードファイルをLLM（大規模言語モデル）のプロンプトに貼り付ける作業を簡略化するCLIツールです。

## 使ってみよう

**例：Reactコンポーネントのリファクタリング相談**

1. プロンプトのテンプレートとなるファイルを作成します（`prompt.txt`）：
```
以下のReactコンポーネントのリファクタリング案を提示してください：
- コンポーネントの分割方針
- カスタムフックの抽出検討
- 状態管理の改善案

#ef src/components/TaskList.tsx
#ef src/components/TaskItem.tsx
#ef src/hooks/useTasks.ts
```

2. コマンドを実行するとテンプレートにファイルを埋め込んだものが出力されます。
```
ef prompt.txt
```

出力例：
```
以下のReactコンポーネントのリファクタリング案を提示してください：
- コンポーネントの分割方針
- カスタムフックの抽出検討
- 状態管理の改善案

src/components/TaskList.tsx
\```tsx
import React from 'react';
import { TaskItem } from './TaskItem';
import { useTasks } from '../hooks/useTasks';

export const TaskList: React.FC = () => {
  const { tasks, toggleTask } = useTasks();
  return (
    <div>
      {tasks.map(task => (
        <TaskItem key={task.id} task={task} onToggle={() => toggleTask(task.id)} />
      ))}
    </div>
  );
};
\```

src/components/TaskItem.tsx
\```tsx
import React from 'react';

...
\```

src/hooks/useTasks.ts
\```typescript
...
\```
```

> pbcopyなどと組み合わせると、すぐ貼り付けられるのでよい：
> ```bash
> # macOS/Linuxの場合
> ef prompt.txt | pbcopy
>
> # Windowsの場合
> ef prompt.txt | clip
> ```

4. Claudeとかに貼ったら、後は祈るだけ。

## インストール方法

1. [Releases](https://github.com/username/embed-files/releases)から、お使いのプラットフォームに対応したバイナリをダウンロードしてください。
   - macOS (Intel): `ef-x86_64-apple-darwin`
   - macOS (Apple Silicon): `ef-aarch64-apple-darwin`
   - Linux: `ef-x86_64-unknown-linux-gnu`
   - Windows: `ef-x86_64-pc-windows-msvc.exe`

2. ダウンロードしたバイナリを、PATHが通っているディレクトリに配置してください。セルフサービスです。

## 使い方

### 基本的な使用方法

```bash
ef <プロンプトテンプレートファイルのパス>
```

例：
```bash
ef prompt.txt
```

### テンプレートファイルの書き方

テンプレートファイル内で以下の指示子を使用することで、ファイルの内容を展開できます：

- `#ef <globパターン>`: globパターンにマッチするファイルを展開
  ```
  #ef src/*.rs      # srcディレクトリ内の全Rustファイル
  #ef lib/**/*.js   # lib以下の全JavaScriptファイル（再帰的）
  ```

- `#efr <正規表現パターン>`: 正規表現パターンにマッチするファイルを展開
  ```
  #efr ^src/.*\.controller\.ts$  # src/以下の全コントローラーファイル
  ```

### 出力形式のカスタマイズ

`.eftemplate`ファイルを作成することで、ファイル展開時の出力形式をカスタマイズできます：

```
ファイル: {filePath}
言語: {language}
==========
{content}
==========
```

利用可能な変数：
- `{filePath}`: ファイルパス
- `{language}`: 強調構文表示用のアレ
- `{content}`: ファイル内容

`.eftemplate`は以下の順序で検索されます：
1. カレントディレクトリ
2. 親ディレクトリ（ルートまで順次）
3. 見つからなかった場合はデフォルトテンプレートを使用

また、filePathは**コマンドを実行した際のカレントディレクトリからの相対パス**に解決されます。
だいたいプロジェクトのルートで実行すると意味が通りやすいかと。

## 自身でのビルド方法

以下のコマンドで各プラットフォーム向けのバイナリをビルドできます：

```bash
# macOS (Intel)
cargo build --release --target x86_64-apple-darwin

# macOS (Apple Silicon)
cargo build --release --target aarch64-apple-darwin

# Linux
cargo build --release --target x86_64-unknown-linux-gnu

# Windows
cargo build --release --target x86_64-pc-windows-msvc
```

rustupでビルド用のなにかを事前に入れる必要あり。
