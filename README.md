# embed-files (ef)

`ef`は、複数のソースコードファイルを、主にチャット型LLMアプリケーションのプロンプトに貼り付ける作業を簡略化することを目的としたCLIツールです。

コードエディタの拡張機能としてそういう機能は確かすでに存在しますが、API呼び出しで従量課金が発生するため、
月額制のブラウザ上のチャットを使いたかったので作りました。

およそ全ての実装をclaudeが行っています。使ったプロンプトの一部はprompts/に格納してあります。

## 使ってみよう

**例：Reactコンポーネントのリファクタリングの相談**

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
\```
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
\```
import React from 'react';

...
\```

src/hooks/useTasks.ts
\```
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

1. [Releases](https://github.com/MoAI522/embed-files/releases)から、お使いのプラットフォームに対応したバイナリをダウンロードしてください。
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
==========
{content}
==========
```

利用可能な変数：
- `{filePath}`: ファイルパス
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
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# macOS (Apple Silicon)
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Linux
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu

# Windows
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```
